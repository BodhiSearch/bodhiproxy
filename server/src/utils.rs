use tokio::signal;

pub async fn shutdown_signal() {
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
