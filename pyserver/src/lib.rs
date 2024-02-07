use ::bodhiproxy as proxy;
use ::bodhiproxy::AppError;
use proxy::build_server_handle;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use std::fmt::Display;

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

impl From<PyAppErr> for PyErr {
  fn from(val: PyAppErr) -> Self {
    match val {
      PyAppErr::AppError(err) => match err {
        AppError::IoError(err) => pyo3::exceptions::PyIOError::new_err(err.to_string()),
        AppError::JoinError(err) => pyo3::exceptions::PyException::new_err(err.to_string()),
        AppError::ServerNotRunning => InvalidServerState::new_err("Server is not running"),
        AppError::ServerRunning => InvalidServerState::new_err("Server is running"),
      },
    }
  }
}

#[pyclass]
struct Server {
  handle: Option<proxy::ServerHandle>,
}

#[allow(clippy::redundant_async_block)]
#[pymethods]
impl Server {
  #[staticmethod]
  #[pyo3(signature = (port = 3000))]
  fn start_server(py: Python, port: u16) -> PyResult<&PyAny> {
    let future = async move {
      let server = build_server_handle(port).await;
      let result: PyResult<Py<PyAny>> = match server {
        Ok(mut handle) => {
          handle.start().await?;
          Python::with_gil(|py| {
            Ok(
              Server {
                handle: Some(handle),
              }
              .into_py(py),
            )
          })
        }
        Err(err) => Err(PyAppErr::AppError(err).into()),
      };
      result
    };
    future_into_py(py, future)
  }

  #[getter]
  fn status(&self) -> PyResult<String> {
    if let Some(ref handle) = self.handle {
      Ok(handle.status())
    } else {
      Ok("stopped".to_string())
    }
  }

  fn stop<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
    if let Some(mut server_handle) = self.handle.take() {
      let future = async move {
        let result: Result<PyObject, PyErr> = match server_handle.stop().await {
          Ok(()) => Python::with_gil(|py| Ok(py.None())),
          Err(err) => Err(err.into()),
        };
        result
      };
      future_into_py(py, future)
    } else {
      Err(PyAppErr::AppError(AppError::ServerNotRunning).into())
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
