use a_sabr::types::Date;
use pyo3::prelude::*;

#[pyclass(name = "AsabrContact")]
pub struct PyAsabrContact {
    #[pyo3(get)]
    pub tx_node: usize,
    #[pyo3(get)]
    pub rx_node: usize,
    #[pyo3(get)]
    pub start_time: Date,
    #[pyo3(get)]
    pub end_time: Date,
}


