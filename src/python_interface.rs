use cpython::{PyResult, CompareOp, ToPyObject, PythonObject};

use crate::{repodata, Version, CompOp};
use crate::repodata::repodata::read_repodata;

impl From<CompareOp> for CompOp {
    fn from(other: CompareOp) -> CompOp {
        match other {
            CompareOp::Eq => CompOp::Eq,
            CompareOp::Ge => CompOp::Ge,
            CompareOp::Le => CompOp::Le,
            CompareOp::Lt => CompOp::Lt,
            CompareOp::Gt => CompOp::Gt,
            CompareOp::Ne => CompOp::Ne
        }
    }
}

py_module_initializer!(libronda, initlibronda, PyInit_libronda, |py, m| {
    m.add(
            py,
            "__doc__",
            "I can haz rusty versions",
        )?;
    m.add_class::<PyVersion>(py)?;
    // m.add(py, "read_repodata", py_fn!(py, read_repodata<'a, P: AsRef<Path>>(path: P)))?;
    Ok(())
});

py_class!(class PyVersion |py| {
    data rust_version: Version;
    def __new__(_cls, arg: &str) -> PyResult<PyVersion> {
        PyVersion::create_instance(py, arg.into())
    }
    def __richcmp__(&self, other: &PyVersion, op: CompareOp) -> PyResult<bool> {
        Ok(self.rust_version(py).compare_to(other.rust_version(py), &op.into()))
    }
    def __repr__(&self) -> PyResult<String> {
        Ok(self.rust_version(py).as_str().to_string())
    }
});

//fn read_repodata_py<'a, P: AsRef<Path>>(_: Python, path: P) -> PyResult<PyObject> {
//    let out = read_repodata(P);
//    Ok(out)
//}