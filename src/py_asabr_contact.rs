use pyo3::prelude::*;
use std::{cell::RefCell, rc::Rc};

use a_sabr::{
    contact::Contact,
    contact_manager::seg::SegmentationManager,
    types::{Date, NodeID},
};

#[pyclass(name = "AsabrContact")]
pub struct PyAsabrContact {
    #[pyo3(get)]
    contact_id: usize,
    #[pyo3(get)]
    tx_node: NodeID,
    #[pyo3(get)]
    rx_node: NodeID,
    #[pyo3(get)]
    start_time: Date,
    #[pyo3(get)]
    end_time: Date,
}

impl PyAsabrContact {
    pub fn from_native_contact(contact: &Rc<RefCell<Contact<SegmentationManager>>>) -> Self {
        let contact_id = Rc::as_ptr(contact) as usize;
        let contact = contact.borrow();

        Self {
            contact_id,
            tx_node: contact.get_tx_node(),
            rx_node: contact.get_rx_node(),
            start_time: contact.info.start,
            end_time: contact.info.end,
        }
    }
}
