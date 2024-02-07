use bodhiproxy::build_server_components;
use rstest::fixture;
use tokio::{sync::oneshot::Sender, task::JoinHandle};

#[fixture]
pub async fn server_handle() -> (u16, JoinHandle<()>, Sender<()>) {
  let port = rand::random::<u16>() % 1000 + 3000;
  let (handle, shutdown) = build_server_components(port)
    .await
    .expect("should build server components");
  let join = tokio::spawn(async {
    handle
      .await
      .expect("error while waiting for server to complete")
  });
  (port, join, shutdown)
}
