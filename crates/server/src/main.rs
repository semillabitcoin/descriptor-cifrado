use std::error::Error;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

const BIND_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_level(true)
        .init();

    std::panic::set_hook(Box::new(|_info| {
        tracing::error!("internal panic");
    }));

    let listener = TcpListener::bind(BIND_ADDR).await?;
    tracing::info!(addr = %BIND_ADDR, "bed-server listening");
    axum::serve(listener, bed_server::router()).await?;
    Ok(())
}
