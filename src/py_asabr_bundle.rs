use pyo3::prelude::*;

use a_sabr::{
    bundle::Bundle,
    types::{Date, Priority, Volume},
};

/// A structure representing a routing bundle containing essential information for pathfinding.
///
/// The `AsabrBundle` struct encapsulates the routing details required for determining optimal paths
/// in a network, including source and destination nodes, priority, size, and expiration time.
///
/// It's a direct mapping of the `a_sabr::bundle::Bundle`
#[pyclass(name = "AsabrBundle")]
#[derive(Debug, Clone)]
pub struct PyAsabrBundle {
    /// The starting node identifier for the routing operation.
    source: usize, // todo
    ///  A vector of node identifiers representing the target destinations for the routing operation.
    pub destinations: Vec<usize>, // todo
    /// The priority level of the bundle, used to influence routing decisions.
    priority: Priority,
    /// The volume size associated with the bundle, which can affect routing constraints.
    size: Volume,
    /// The expiration date for the bundle.
    expiration: Date,
}

#[pymethods]
impl PyAsabrBundle {
    #[new]
    fn new(
        source: usize, // todo
        destinations: Vec<usize>, // todo
        priority: Priority,
        size: Volume,
        expiration: Date,
    ) -> Self {
        Self {
            source,
            destinations,
            priority,
            size,
            expiration,
        }
    }
}

impl PyAsabrBundle {
    pub fn to_native_bundle(&self) -> Bundle {
        Bundle {
            priority: self.priority,
            size: self.size,
            expiration: self.expiration,
        }
    }
}
