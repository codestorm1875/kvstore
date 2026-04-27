use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;

mod handler;
mod protocol;
mod store;

use handler::handle_client;
use store::KvStore;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let store = KvStore::new();
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    info!("KVStore listening on 127.0.0.1:6379");

    // Spawn cleanup task to remove expired keys
    let cleanup_store = store.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_store.cleanup_expired().await;
            info!("Cleaned up expired keys");
        }
    });

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);

        let store = store.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, store).await {
                tracing::error!("Error handling client {}: {}", addr, e);
            }
        });
    }
}
