use crate::{build_server, AppError, ServerHandle, TOKIO_RUNTIME};
use tokio::sync::oneshot;

pub struct Server {
  pub port: u16,
  shutdown: Option<oneshot::Sender<()>>,
  handle: Option<tokio::task::JoinHandle<std::result::Result<(), std::io::Error>>>,
}

impl Drop for Server {
  fn drop(&mut self) {
    if let Some(shutdown) = self.shutdown.take() {
      if shutdown.send(()).is_err() {
        eprintln!("Error sending shutdown signal");
      }
      if let Some(join_handle) = self.handle.take() {
        if let Err(e) = TOKIO_RUNTIME.block_on(join_handle) {
          eprintln!("Error shutting down server: {}", e);
        }
      }
    }
  }
}

impl Server {
  pub fn new(port: u16) -> Result<Self, AppError> {
    let ServerHandle {
      port,
      shutdown,
      handle,
    } = TOKIO_RUNTIME.block_on(build_server(port))?;
    let join_handle = TOKIO_RUNTIME.spawn(async move { handle.await });
    Ok(Server {
      port,
      shutdown: Some(shutdown),
      handle: Some(join_handle),
    })
  }

  pub fn status(&self) -> String {
    if self.shutdown.is_some() {
      "running".to_string()
    } else {
      "stopped".to_string()
    }
  }

  pub fn stop(&mut self) -> Result<(), AppError> {
    if let Some(shutdown) = self.shutdown.take() {
      if shutdown.send(()).is_err() {
        eprintln!("Error sending shutdown signal");
      }
      if let Some(join_handle) = self.handle.take() {
        let r = TOKIO_RUNTIME.block_on(join_handle)?;
        Ok(r?)
      } else {
        Err(AppError::ServerNotRunning)
      }
    } else {
      Err(AppError::ServerNotRunning)
    }
  }
}
