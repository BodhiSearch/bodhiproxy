use axum::routing::get;
use axum::Router;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::oneshot;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
  IoError(#[from] std::io::Error),
}

impl Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AppError::IoError(e) => write!(f, "IO error: {}", e),
    }
  }
}

pub struct ServerHandle {
  pub port: u16,
  pub shutdown: oneshot::Sender<()>,
  pub handle: axum::serve::WithGracefulShutdown<Router, Router, ShutdownWrapper>,
}

pub async fn build_server(port: u16) -> Result<ServerHandle, AppError> {
  let app: Router = axum::Router::new()
    .route("/", get(|| async { "world hell" }))
    .layer((
      TraceLayer::new_for_http(),
      TimeoutLayer::new(Duration::from_secs(5)),
    ));
  let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
  let (shutdown, rx) = oneshot::channel::<()>();
  let handle = axum::serve(listener, app).with_graceful_shutdown(ShutdownWrapper { rx });
  Ok(ServerHandle {
    port,
    shutdown,
    handle,
  })
}

pub struct ShutdownWrapper {
  rx: tokio::sync::oneshot::Receiver<()>,
}

impl Future for ShutdownWrapper {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match Pin::new(&mut self.rx).poll(cx) {
      Poll::Ready(_) => Poll::Ready(()),
      Poll::Pending => Poll::Pending,
    }
  }
}
