use bodhiproxy::utils::shutdown_signal;
use bodhiproxy::{spawn_server, AppError};

#[tokio::main]
async fn main() -> Result<(), AppError> {
  spawn_server(3000, async {
    shutdown_signal().await;
  })
  .await
}
