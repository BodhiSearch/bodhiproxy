pub mod utils;

use axum::routing::get;
use axum::serve::WithGracefulShutdown;
use axum::Router;
use pyo3::PyErr;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::oneshot::{self, Sender};
use tokio::task::{JoinError, JoinHandle};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
  IoError(#[from] std::io::Error),
  JoinError(#[from] JoinError),
  ServerNotRunning,
  ServerRunning,
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
      AppError::ServerRunning => write!(f, "Server is already running"),
    }
  }
}

pub struct ServerHandle {
  pub port: u16,
  pub shutdown: Option<oneshot::Sender<()>>,
  pub handle: Option<WithGracefulShutdown<Router, Router, ShutdownWrapper>>,
  pub join: Option<JoinHandle<Result<(), AppError>>>,
}

pub async fn build_server_components(
  port: u16,
) -> Result<
  (
    WithGracefulShutdown<Router, Router, ShutdownWrapper>,
    Sender<()>,
  ),
  AppError,
> {
  let app: Router = route();
  let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
  let (shutdown, rx) = oneshot::channel::<()>();
  let handle = axum::serve(listener, app).with_graceful_shutdown(ShutdownWrapper { rx });
  Ok((handle, shutdown))
}

pub async fn spawn_server(
  port: u16,
  shutdown_fut: impl Future<Output = ()> + Send + 'static,
) -> Result<(), AppError> {
  let (handle, shutdown) = build_server_components(port).await?;
  tokio::spawn(async {
    shutdown_fut.await;
    shutdown.send(()).unwrap();
  });
  handle.await.map_err(AppError::IoError)
}

pub async fn build_server_handle(port: u16) -> Result<ServerHandle, AppError> {
  let (handle, shutdown) = build_server_components(port).await?;
  Ok(ServerHandle {
    port,
    shutdown: Some(shutdown),
    handle: Some(handle),
    join: None,
  })
}

impl ServerHandle {
  pub async fn start(&mut self) -> Result<(), AppError> {
    if let Some(handler) = self.handle.take() {
      let join: JoinHandle<Result<(), AppError>> = tokio::spawn(async {
        handler.await.map_err(AppError::IoError)?;
        Ok(())
      });
      self.join = Some(join);
      Ok(())
    } else {
      Err(AppError::ServerRunning)
    }
  }

  pub fn status(&self) -> String {
    if self.handle.is_some() {
      return "built".to_string();
    }
    if self.join.is_some() {
      "running".to_string()
    } else {
      "stopped".to_string()
    }
  }

  pub async fn stop(&mut self) -> Result<(), AppError> {
    if let Some(shutdown) = self.shutdown.take() {
      let _ = shutdown.send(());
    } else {
      return Err(AppError::ServerNotRunning);
    }
    if let Some(join) = self.join.take() {
      let _ = join.await;
      Ok(())
    } else {
      Err(AppError::ServerNotRunning)
    }
  }
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
