use pyo3::{exceptions::PyBaseException, prelude::*};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use a_sabr::{
    contact_manager::seg::SegmentationManager,
    contact_plan::from_tvgutil_file::TVGUtilContactPlan,
    node::Node,
    node_manager::none::NoManagement,
    route_storage::cache::TreeCache,
    routing::{aliases::SpsnMpt, spsn::Spsn},
    types::{Date, NodeID},
};

use crate::{py_asabr_bundle::PyAsabrBundle, py_asabr_contact::PyAsabrContact};

// NOT thread-safe
#[pyclass(name = "AsabrRouter", unsendable)]
pub struct PyAsabrRouter {
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
    fn new(tvgutil_contact_plan_filepath: &str) -> PyResult<Self> {
        let contact_plan =
            TVGUtilContactPlan::parse::<SegmentationManager>(tvgutil_contact_plan_filepath);

        match contact_plan {
            Ok((nodes, contacts)) => {
                let nodes_id_map = make_nodes_id_map(&nodes);

                let cache = Rc::new(RefCell::new(TreeCache::new(false, false, 10)));
                let router = Spsn::new(nodes, contacts, cache, false);

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
        let result = self.nodes_id_map.get(node_name);

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
