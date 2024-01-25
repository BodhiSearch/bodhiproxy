use bodhiproxy::utils::shutdown_signal;
use bodhiproxy::{build_server, AppError};

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
