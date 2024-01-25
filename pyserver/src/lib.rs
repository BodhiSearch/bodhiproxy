use ::bodhiproxy::AppError;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use std::fmt::Display;
use ::bodhiproxy as proxy;

#[derive(thiserror::Error, Debug)]
pub enum PyAppErr {
  AppError(#[from] ::bodhiproxy::AppError),
}

impl Display for PyAppErr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PyAppErr::AppError(e) => write!(f, "App error: {}", e),
    }
  }
}

#[pyclass]
struct Server {
  server: proxy::Server,
}

#[allow(clippy::redundant_async_block)]
#[pymethods]
impl Server {
  #[new]
  #[pyo3(signature = (port = 3000))]
  fn new(port: u16) -> PyResult<Self> {
    let server = proxy::Server::new(port)?;
    Ok(Server { server })
  }

  #[getter]
  fn status(&self) -> PyResult<String> {
    Ok(self.server.status())
  }

  fn stop(&mut self) -> PyResult<()> {
    match self.server.stop() {
      Ok(_) => Ok(()),
      Err(AppError::ServerNotRunning) => Err(InvalidServerState::new_err("Server is not running")),
      Err(e) => Err(e.into()),
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
