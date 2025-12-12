use redis::{
    AsyncCommands, RedisError,
    streams::{StreamReadOptions, StreamReadReply},
};
use std::{env, time::Duration};
use tokio::time::sleep;

use crate::utils::smtp::send_invitation;

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
                        let result = send_invitation(
                            id, client_url.clone(), sender_first_name, sender_last_name, receiver
                        ).await;

                        if let Err(e) = result {
                            tracing::error!("Ошибка отправки {}", e);
                            continue;
                        }

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
