mod utils;
use std::time::Duration;

use bodhiproxy::AppError;
use rstest::rstest;
use tokio::{sync::oneshot::Sender, task::JoinHandle};
use utils::server_handle;

#[rstest]
#[timeout(Duration::from_millis(100))]
#[tokio::test]
async fn test_server_ping(
  #[future] server_handle: (u16, JoinHandle<()>, Sender<()>),
) -> Result<(), AppError> {
  let (port, join, shutdown) = server_handle.await;
  let client = reqwest::Client::new();
  let url = format!("http://localhost:{}/ping", port);
  let response = client.get(&url).send().await.unwrap();
  assert!(response.status().is_success());
  assert_eq!(response.text().await.unwrap(), "pong");
  shutdown.send(()).expect("should send shutdown signal");
  join.await?;
  Ok(())
}
