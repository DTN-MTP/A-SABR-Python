use pyo3::prelude::*;

mod py_asabr_router;
use py_asabr_router::PyAsabrRouter;

mod py_asabr_contact;
use py_asabr_contact::PyAsabrContact;

mod py_asabr_bundle;
use py_asabr_bundle::PyAsabrBundle;

#[pymodule]
fn a_sabr_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAsabrRouter>()?;
    m.add_class::<PyAsabrContact>()?;
    m.add_class::<PyAsabrBundle>()?;
    Ok(())
}
