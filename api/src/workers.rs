use redis::{AsyncCommands, streams::StreamReadOptions};
use lettre::{Message, AsyncSmtpTransport, Tokio1Executor, Transport};

pub async fn invitation_worker(redis_client: redis::Client) {
    let mut redis_con = redis_client.get_multiplexed_async_connection().await.unwrap();

    loop {

        let mut options = StreamReadOptions::default();
        options.block(5000);
        options.count(10);

        let ivitations: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = 
            redis_con.xread_options(&["invitations_stream"], &["0"], &options).await.unwrap_or_default();

        for (_stream_name, msgs) in ivitations {
            for (_id, fields) in msgs {
                let mut link_id = String::new();
                let mut receiver = String::new();
                let mut sender_first_name = String::new();
                let mut sender_last_name = String::new();

                for (field, value) in fields {
                    match field.as_str() {
                        "link_id" => link_id = value,
                        "receiver" => receiver = value,
                        "sender_first_name" => sender_first_name = value,
                        "sender_last_name" => sender_last_name = value,
                        _ => {}
                    }
                }

                println!("Отправляем приглашение {} → {} ({}, {})", link_id, receiver, sender_first_name, sender_last_name);
                // Здесь вызываем send_email или другую функцию отправки письма
            }
        }
    }
}