use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    AsyncCommands,
};

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
                        tracing::error!("Failed to acknowledge message {}: {}", msg_id, e);
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    }
}
