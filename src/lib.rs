use std::{cell::RefCell, collections::HashMap, rc::Rc};

use a_sabr::{
    bundle::Bundle,
    contact::Contact,
    contact_manager::seg::SegmentationManager,
    contact_plan::{asabr_file_lexer::FileLexer, from_asabr_lexer::ASABRContactPlan},
    node::Node,
    node_manager::none::NoManagement,
    route_storage::cache::TreeCache,
    routing::{aliases::SpsnMpt, spsn::Spsn},
    types::{Date, NodeID, Priority, Volume},
};
use pyo3::{exceptions::PyBaseException, prelude::*};

#[pyclass(name = "AsabrContact")]
struct PyAsabrContact {
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
    fn from_native_contact(contact: &Rc<RefCell<Contact<SegmentationManager>>>) -> Self {
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

/// A structure representing a routing bundle containing essential information for pathfinding.
///
/// The `AsabrBundle` struct encapsulates the routing details required for determining optimal paths
/// in a network, including source and destination nodes, priority, size, and expiration time.
///
/// It's a direct mapping of the `a_sabr::bundle::Bundle`
#[pyclass(name = "AsabrBundle")]
#[derive(Debug, Clone)]
struct PyAsabrBundle {
    /// The starting node identifier for the routing operation.
    source: NodeID,
    ///  A vector of node identifiers representing the target destinations for the routing operation.
    destinations: Vec<NodeID>,
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
        source: NodeID,
        destinations: Vec<NodeID>,
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
    fn to_native_bundle(&self) -> Bundle {
        Bundle {
            source: self.source,
            destinations: self.destinations.clone(),
            priority: self.priority,
            size: self.size,
            expiration: self.expiration,
        }
    }
}

// NOT thread-safe
#[pyclass(name = "AsabrRouter", unsendable)]
struct PyAsabrRouter {
    nodes_id_map: HashMap<String, NodeID>,

    router: SpsnMpt<NoManagement, SegmentationManager>,
}

fn make_nodes_id_map(nodes: &Vec<Node<NoManagement>>) -> HashMap<String, NodeID> {
    let mut nodes_id_map = HashMap::new();

    for node in nodes {
        nodes_id_map.insert(node.get_node_name(), node.get_node_id());
    }

    nodes_id_map
}

#[pymethods]
impl PyAsabrRouter {
    #[new]
    fn new(contact_plan_filepath: &str) -> PyResult<Self> {
        match FileLexer::new(contact_plan_filepath) {
            Ok(mut lexer) => {
                let mut contact_plan = ASABRContactPlan::new();
                let res =
                    contact_plan.parse::<NoManagement, SegmentationManager>(&mut lexer, None, None);

                match res {
                    Ok((nodes, contacts)) => {
                        //println!("Contacts:\n{:?}\nNodes:\n{:?}", contacts.len(), nodes.len());
                        let nodes_id_map = make_nodes_id_map(&nodes);
                        //println!("Node ID map: {:?}", nodes_id_map);
                        let cache = Rc::new(RefCell::new(TreeCache::new(false, false, 10)));

                        let router = Spsn::new(nodes, contacts, cache, false);
                        //println!("{:?}", router.pathfinding.get_multigraph());

                        Ok(Self {
                            nodes_id_map,
                            router,
                        })
                    }
                    Err(err) => Err(PyErr::new::<PyBaseException, _>(format!(
                        "[A-SABR][ContactPlan] Parse error: {}",
                        err
                    ))),
                }
            }
            Err(err) => Err(PyErr::new::<PyBaseException, _>(format!(
                "[A-SABR][ContactPlan] Open error: {}",
                err
            ))),
        }
    }

    fn route(
        &mut self,
        source: NodeID,
        bundle: PyAsabrBundle,
        curr_time: Date,
        excluded_nodes: Vec<NodeID>,
    ) -> Vec<(PyAsabrContact, Vec<NodeID>)> {
        let bundle = bundle.to_native_bundle();

        if let Some(routing_output) = self
            .router
            .route(source, &bundle, curr_time, &excluded_nodes)
        {
            let mut py_routing_output = Vec::new();

            for (_, (contact, reachable_nodes)) in &routing_output.first_hops {
                py_routing_output.push((
                    PyAsabrContact::from_native_contact(contact),
                    reachable_nodes.clone(),
                ));
            }

            py_routing_output
        } else {
            Vec::new()
        }
    }

    fn get_node_id(&self, node_name: &str) -> PyResult<NodeID> {
        let canonical_node_name = node_name.replace(" ", "_");
        let result = self.nodes_id_map.get(&canonical_node_name);

        if let Some(node_id) = result {
            Ok(*node_id)
        } else {
            Err(PyErr::new::<PyBaseException, _>(format!(
                "Node '{}' unknown",
                node_name
            )))
        }
    }
}

#[pymodule]
fn a_sabr_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAsabrBundle>()?;
    m.add_class::<PyAsabrContact>()?;
    m.add_class::<PyAsabrRouter>()?;
    Ok(())
}
