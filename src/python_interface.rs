extern crate pyo3;

use pyo3::py::modinit;
use pyo3::{Python, PyResult, PyModule, exc};

use crate::repodata;

#[modinit(libronda)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    fn load_repodata(file_path: String) -> PyResult {
        repodata::
    }
}