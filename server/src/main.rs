use bodhiproxy::{build_server, AppError};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), AppError> {
  let server_handle = build_server(3000).await?;
  tokio::spawn(async move {
    shutdown_signal().await;
    server_handle.shutdown.send(()).unwrap();
  });
  server_handle.handle.await?;
  Ok(())
}

async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
    eprintln!("Received Ctrl+C, stopping server");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
    eprintln!("Received SIGTERM, stopping server");
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();
  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }
}
