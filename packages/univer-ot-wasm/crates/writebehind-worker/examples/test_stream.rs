//! Test Redis Streams connectivity
//!
//! Run with: cargo run -p writebehind-worker --example test_stream

use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    let _ = dotenvy::dotenv();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    println!("Connecting to Redis: {}", redis_url);

    let client = redis::Client::open(redis_url.as_str())?;
    println!("Client created");

    // Test basic connection
    let mut conn = client.get_multiplexed_async_connection().await?;
    println!("Connection established");

    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
    println!("PING response: {}", pong);

    // Test Redis Streams
    println!("\n=== Testing Redis Streams ===");

    let stream_key = "ot:{wb}:stream";
    let group_name = "writebehind-workers";
    let consumer_name = "test-consumer";

    // Create consumer group (ignore error if already exists)
    let result: Result<String, _> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(stream_key)
        .arg(group_name)
        .arg("0")
        .arg("MKSTREAM")
        .query_async(&mut conn)
        .await;
    match result {
        Ok(_) => println!("Consumer group '{}' created", group_name),
        Err(e) if e.to_string().contains("BUSYGROUP") => {
            println!("Consumer group '{}' already exists", group_name);
        }
        Err(e) => return Err(e.into()),
    }

    // Add a test message
    let msg_id: String = redis::cmd("XADD")
        .arg(stream_key)
        .arg("MAXLEN")
        .arg("~")
        .arg("100000")
        .arg("*")
        .arg("doc_id")
        .arg("test-doc-123")
        .query_async(&mut conn)
        .await?;
    println!("Added test message: {}", msg_id);

    // Read messages with XREADGROUP
    println!("\nReading messages with XREADGROUP...");
    let result: redis::Value = redis::cmd("XREADGROUP")
        .arg("GROUP")
        .arg(group_name)
        .arg(consumer_name)
        .arg("COUNT")
        .arg("10")
        .arg("BLOCK")
        .arg("2000")
        .arg("STREAMS")
        .arg(stream_key)
        .arg(">")
        .query_async(&mut conn)
        .await?;

    println!("XREADGROUP result: {:?}", result);

    // Get stream info
    println!("\n=== Stream Info ===");
    let info: redis::Value = redis::cmd("XINFO")
        .arg("STREAM")
        .arg(stream_key)
        .query_async(&mut conn)
        .await?;
    println!("XINFO STREAM: {:?}", info);

    let groups: redis::Value = redis::cmd("XINFO")
        .arg("GROUPS")
        .arg(stream_key)
        .query_async(&mut conn)
        .await?;
    println!("XINFO GROUPS: {:?}", groups);

    // ACK the message
    let acked: u64 = redis::cmd("XACK")
        .arg(stream_key)
        .arg(group_name)
        .arg(&msg_id)
        .query_async(&mut conn)
        .await?;
    println!("\nACKed {} message(s)", acked);

    // Wait and listen for more messages
    println!("\nWaiting for new messages (send XADD ot:{{wb}}:stream * doc_id <id> from redis-cli)...\n");

    loop {
        tokio::select! {
            result = async {
                redis::cmd("XREADGROUP")
                    .arg("GROUP")
                    .arg(group_name)
                    .arg(consumer_name)
                    .arg("COUNT")
                    .arg("10")
                    .arg("BLOCK")
                    .arg("5000")
                    .arg("STREAMS")
                    .arg(stream_key)
                    .arg(">")
                    .query_async::<redis::Value>(&mut conn)
                    .await
            } => {
                match result {
                    Ok(redis::Value::Nil) => {
                        println!("... still waiting (5s tick)");
                    }
                    Ok(value) => {
                        println!(">>> Received: {:?}", value);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
}
