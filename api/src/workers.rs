use lettre::{AsyncSmtpTransport, Message, Tokio1Executor, Transport};
use redis::{AsyncCommands, streams::StreamReadOptions};

pub async fn invitation_worker(redis_client: redis::Client) {
    let mut redis_con = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let mut last_seen_id = "0".to_string();

    loop {
        let options = StreamReadOptions::default().block(5000).count(10);

        let invitations: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = redis_con
            .xread_options(&["invitations_stream"], &[&last_seen_id], &options)
            .await
            .unwrap_or_default();

        for (_stream_name, msgs) in invitations {
            for (msg_id, fields) in msgs {
                let mut id = String::new();
                let mut receiver = String::new();
                let mut sender_first_name = String::new();
                let mut sender_last_name = String::new();

                for (field, value) in fields {
                    match field.as_str() {
                        "id" => id = value,
                        "receiver" => receiver = value,
                        "sender_first_name" => sender_first_name = value,
                        "sender_last_name" => sender_last_name = value,
                        _ => {}
                    }
                }

                // * Очередь: Redis Streams с Группами Потребителей
                //   (Consumer Groups).
                //     * Эта технология предоставляет всё необходимое из
                //       коробки: XREADGROUP (взять задачу), список
                //       ожидающих (делает ее "невидимой") и XACK
                //       (подтвердить выполнение). Это идеальный инструмент
                //       для "at-least-once" доставки.
                tracing::debug!(
                    "Отправляем приглашение {} → {} ({}, {})",
                    id,
                    receiver,
                    sender_first_name,
                    sender_last_name
                );

                last_seen_id = msg_id;
            }
        }
    }
}
