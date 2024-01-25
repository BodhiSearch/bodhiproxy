mod utils;
use std::error::Error;

use bodhiproxy::Server;
use rstest::rstest;
use utils::server;

#[rstest]
#[tokio::test]
async fn test_server_ping(#[future] server: Server) -> Result<(), Box<dyn Error>> {
  let server = server.await;
  let client = reqwest::Client::new();
  let url = format!("http://localhost:{}/ping", server.port);
  let response = client.get(&url).send().await.unwrap();
  assert!(response.status().is_success());
  assert_eq!(response.text().await.unwrap(), "pong");
  Ok(())
}
