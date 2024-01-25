pub mod server;
pub mod utils;
pub use server::*;

use axum::routing::get;
use axum::Router;
use once_cell::sync::Lazy;
use pyo3::PyErr;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::task::JoinError;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
});

#[derive(thiserror::Error, Debug)]
pub enum AppError {
  IoError(#[from] std::io::Error),
  JoinError(#[from] JoinError),
  ServerNotRunning,
}

impl From<AppError> for PyErr {
  fn from(err: AppError) -> Self {
    pyo3::exceptions::PyIOError::new_err(err.to_string())
  }
}

impl Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AppError::IoError(e) => write!(f, "IO error: {}", e),
      AppError::JoinError(e) => write!(f, "Join error: {}", e),
      AppError::ServerNotRunning => write!(f, "Server is not running"),
    }
  }
}

pub struct ServerHandle {
  pub port: u16,
  pub shutdown: oneshot::Sender<()>,
  pub handle: axum::serve::WithGracefulShutdown<Router, Router, ShutdownWrapper>,
}

pub async fn build_server(port: u16) -> Result<ServerHandle, AppError> {
  let app: Router = route();
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

fn route() -> Router {
  axum::Router::new()
    .route("/ping", get(|| async { "pong" }))
    .layer((
      TraceLayer::new_for_http(),
      TimeoutLayer::new(Duration::from_secs(5)),
    ))
}
