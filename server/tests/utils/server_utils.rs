use bodhiproxy::Server;
use rstest::fixture;

#[fixture]
pub async fn server() -> Server {
  let port = rand::random::<u16>() % 1000 + 3000;
  Server::new_async(port)
    .await
    .expect("failed to start server")
}
