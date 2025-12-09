use anyhow::Ok as anyOk;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use redis::{
    AsyncCommands, RedisError,
    streams::{StreamReadOptions, StreamReadReply},
};
use serde_json::json;
use std::{env, time::Duration};
use tera::{Context, Tera};
use tokio::time::sleep;

pub const INVITATIONS_STREAM_NAME: &str = "invitations_stream";
const GROUP_NAME: &str = "invitations_group";
const CONSUMER_NAME: &str = "consumer-1";

pub async fn invitation_worker(redis_client: redis::Client) {
    let mut redis_con = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let _: () = redis_con
        .xgroup_create_mkstream(INVITATIONS_STREAM_NAME, GROUP_NAME, "$")
        .await
        .unwrap_or_default();

    let options = StreamReadOptions::default()
        .group(GROUP_NAME, CONSUMER_NAME)
        .count(10);

    let client_url = env::var("CLIENT_URL").unwrap();
    let smtp_host = env::var("SMTP_HOST").unwrap();
    let smtp_from = env::var("SMTP_FROM").unwrap();

    let mailer = if cfg!(debug_assertions) {
        let smtp_user = env::var("SMTP_USER").unwrap();
        let smtp_password = env::var("SMTP_PASSWORD").unwrap();

        let creds = Credentials::new(smtp_user, smtp_password);

        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
            .unwrap()
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_host).build()
    };

    loop {
        let results: [Result<StreamReadReply, RedisError>; 2] = [
            redis_con
                .xread_options(&[INVITATIONS_STREAM_NAME], &["0"], &options)
                .await,
            redis_con
                .xread_options(&[INVITATIONS_STREAM_NAME], &[">"], &options)
                .await,
        ];

        for result in results {
            if let Ok(reply) = result {
                for stream_key in reply.keys {
                    for msg in stream_key.ids {
                        let msg_id = &msg.id;
                        let mut id = String::new();
                        let mut receiver = String::new();
                        let mut sender_first_name = String::new();
                        let mut sender_last_name = String::new();

                        for (field, value) in &msg.map {
                            if let Ok(value_str) = redis::from_redis_value::<String>(value.clone())
                            {
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
                            sender_first_name, sender_last_name
                        );

                        let notification = json!({
                            "consumerEmail": receiver,
                            "title": "Приглашение на регистрацию",
                            "message": invitation_text,
                            "link": format!(
                                "{}/auth/registration?code={}",
                                client_url,
                                id
                            ),
                            "buttonName": "Зарегистрироваться"
                        });

                        let result = (async {
                            let tera = Tera::new("api/templates/**/*")?;
                            let mut ctx = Context::new();
                            ctx.insert("notification", &notification);
                            let html = tera.render("notification.html", &ctx)?;

                            let email = Message::builder()
                                .from(smtp_from.parse().unwrap())
                                .to(receiver.parse().unwrap())
                                .subject("Приглашение на регистрацию")
                                .header(ContentType::TEXT_HTML)
                                .body(html)?;

                            mailer.send(email).await?;
                            anyOk(())
                        })
                        .await;

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
                            .xack(INVITATIONS_STREAM_NAME, GROUP_NAME, &[msg_id])
                            .await;

                        if let Err(e) = ack_result {
                            tracing::error!("Ошибка при ack {}: {}", msg_id, e);
                        }
                    }
                }
            }
        }
        sleep(Duration::from_secs(15)).await;
    }
}
