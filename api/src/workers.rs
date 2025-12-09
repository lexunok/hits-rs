use anyhow::Ok as anyOk;
use lettre::{AsyncSmtpTransport, Message, Tokio1Executor, AsyncTransport, message::header::ContentType};
use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    AsyncCommands,
};
use serde_json::json;
use tera::{Context, Tera};

const STREAM_NAME: &str = "invitations_stream";
const GROUP_NAME: &str = "invitations_group";
const CONSUMER_NAME: &str = "consumer-1";

pub async fn invitation_worker(redis_client: redis::Client) {
    let mut redis_con = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let _: () = redis_con
        .xgroup_create_mkstream(STREAM_NAME, GROUP_NAME, "$")
        .await
        .unwrap_or_default();
    
    loop {
        let options = StreamReadOptions::default()
            .group(GROUP_NAME, CONSUMER_NAME)
            .count(10);

        let result: Result<StreamReadReply, _> = redis_con
            .xread_options(&[STREAM_NAME], &[">"], &options)
            .await;

        if let Ok(reply) = result {
            for stream_key in reply.keys {
                for msg in stream_key.ids {
                    let msg_id = &msg.id;
                    let mut id = String::new();
                    let mut receiver = String::new();
                    let mut sender_first_name = String::new();
                    let mut sender_last_name = String::new();

                    for (field, value) in &msg.map {
                        if let Ok(value_str) = redis::from_redis_value::<String>(value.clone()) {
                            match field.as_str() {
                                "id" => id = value_str,
                                "receiver" => receiver = value_str,
                                "sender_first_name" => sender_first_name = value_str,
                                "sender_last_name" => sender_last_name = value_str,
                                _ => {}
                            }
                        }
                    }
                    let invitation_text = format!(
                            "Вас пригласил(-а) зарегистрироваться на портал HITS {} {} \
                            в качестве пользователя. Для регистрации на сервисе \
                            перейдите по данной ссылке и заполните все поля.",
                            sender_first_name,
                            sender_last_name
                        );

                    let notification = json!({
                        "consumerEmail": receiver,
                        "title": "Приглашение на регистрацию",
                        "message": invitation_text,
                        "link": format!(
                            //ЗАМЕНИТЬ НА HITS или DEV
                            "https://authorization.example.com/auth/registration?code={}",
                            id
                        ),
                        "buttonName": "Зарегистрироваться"
                    });

                    let result = (async {
                        let tera = Tera::new("templates/**/*")?;
                        let mut ctx = Context::new();
                        ctx.insert("notification", &notification);
                        let html = tera.render("notification.html", &ctx)?;

                        let email = Message::builder()
                            .from("hist@tyuiu.ru".parse().unwrap())
                            .to(receiver.parse().unwrap())
                            .subject("Приглашение на регистрацию")
                            .header(ContentType::TEXT_HTML)
                            .body(html)?;

                        //ЗАМЕНИТЬ НА ПРОД/ДЕВ
                        let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("smtp.tsogu.ru").build();

                        mailer.send(email).await?;
                        anyOk(())
                    }).await;

                    if let Err(e) = result {
                        tracing::error!("Ошибка отправки {}", e);
                        continue;
                    }

                    tracing::debug!(
                        "Отправляем приглашение {} → {} ({}, {})",
                        id,
                        receiver,
                        sender_first_name,
                        sender_last_name
                    );

                    let ack_result: Result<i64, _> = redis_con
                        .xack(STREAM_NAME, GROUP_NAME, &[msg_id])
                        .await;

                    if let Err(e) = ack_result {
                        tracing::error!("Ошибка при ack {}: {}", msg_id, e);
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    }
}
