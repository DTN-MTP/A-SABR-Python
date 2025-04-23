use pyo3::{exceptions::PyBaseException, prelude::*};
use std::collections::HashMap;

use a_sabr::{
    bundle::Bundle,
    contact_manager::{seg::SegmentationManager, ContactManager},
    contact_plan::from_tvgutil_file::TVGUtilContactPlan,
    node::Node,
    node_manager::none::NoManagement,
    routing::{aliases::*, Router, RoutingOutput},
    types::{Date, NodeID},
};

use crate::{py_asabr_bundle::PyAsabrBundle, py_asabr_contact::PyAsabrContact};

// NOT thread-safe
#[pyclass(name = "AsabrRouter", unsendable)]
pub struct PyAsabrRouter {
    nodes_id_map: HashMap<String, NodeID>,
    router: Box<dyn Router<NoManagement, SegmentationManager>>,
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
    fn new(tvgutil_contact_plan_filepath: &str, router_type: &str) -> PyResult<Self> {
        let contact_plan = TVGUtilContactPlan::parse::<NoManagement, SegmentationManager>(
            tvgutil_contact_plan_filepath,
        );

        match contact_plan {
            Ok((nodes, contacts)) => {
                let nodes_id_map = make_nodes_id_map(&nodes);
                let router = build_generic_router::<NoManagement, SegmentationManager>(
                    router_type,
                    nodes,
                    contacts,
                    Some(SpsnOptions {
                        check_priority: false,
                        check_size: false,
                        max_entries: 10,
                    }),
                );

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
