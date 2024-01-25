use std::net::Shutdown;

use ::bodhiproxy::{build_server, ServerHandle};
use once_cell::sync::Lazy;
use pyo3::{create_exception, exceptions::PyException, prelude::*};
use tokio::runtime::Runtime;

static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
});

#[pyclass]
struct Server {
  #[pyo3(get)]
  port: u16,
  shutdown: Option<tokio::sync::oneshot::Sender<()>>,
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

#[allow(clippy::redundant_async_block)]
#[pymethods]
impl Server {
  #[new]
  #[pyo3(signature = (port = 3000))]
  fn new(port: u16) -> PyResult<Self> {
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

  #[getter]
  fn status(&self) -> PyResult<String> {
    if self.shutdown.is_some() {
      Ok("running".to_string())
    } else {
      Ok("stopped".to_string())
    }
  }

  fn stop(&mut self) -> PyResult<()> {
    if let Some(shutdown) = self.shutdown.take() {
      shutdown.send(()).unwrap();
      if let Some(join_handle) = self.handle.take() {
        tokio::runtime::Runtime::new()
          .unwrap()
          .block_on(join_handle)
          .unwrap()?
      }
      Ok(())
    } else {
      Err(InvalidServerState::new_err("Server is not running"))
    }
  }
}

create_exception!(bodhiproxy, InvalidServerState, PyException);

#[pymodule]
fn bodhiproxy(_py: Python, m: &PyModule) -> PyResult<()> {
  m.add(
    "InvalidServerState",
    m.py().get_type::<InvalidServerState>(),
  )?;
  m.add_class::<Server>()?;
  Ok(())
}
