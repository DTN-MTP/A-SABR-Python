use std::{collections::HashMap, fs::File};
use pyo3::{exceptions::{PyBaseException, PyValueError}, prelude::*};

use a_sabr::{
    contact_manager::segmentation::seg::SegmentationManager, contact_plan::{RealNode, from_tvgutil_file::TVGUtilContactPlan}, errors::ASABRError, mk_router, multigraph::{Multigraph, NodeRef, RoutableNodeRef}, node_manager::none::NoManagement, pathfinding::Pathfinding, types::{Date, NodeID}, utils::{Router, make_guard}, vnode::VirtualNodeInfo,
};

use crate::{py_asabr_bundle::PyAsabrBundle, py_asabr_contact::PyAsabrContact};

/// Extension trait to easily convert ASABRError results into PyResult.
pub trait IntoPyResult<T> {
    fn into_py_res(self) -> PyResult<T>;
}

impl<T> IntoPyResult<T> for Result<T, ASABRError> {
    fn into_py_res(self) -> PyResult<T> {
        self.map_err(|err| PyValueError::new_err(format!("[A-SABR Error]: {:?}", err)))
    }
}

#[pyclass(name = "AsabrRouter", unsendable)]
pub struct PyAsabrRouter {
    nodes_id_map: HashMap<String, NodeID>,
    router: Router<
        'static,
        NoManagement,
        SegmentationManager,
        Box<
            dyn Pathfinding<
                'static,
                NoManagement,
                SegmentationManager,
                RoutableNodeRef<'static>,
            > + 'static,
        >,
        RoutableNodeRef<'static>,
    >,
}

fn make_nodes_id_map(
    real_nodes: &[RealNode<NoManagement>],
    _vnodes: &[VirtualNodeInfo],
) -> HashMap<String, NodeID> {
    let mut nodes_id_map = HashMap::new();

    for vertex in real_nodes {
        match vertex {
            RealNode::Inode(node) | RealNode::Enode(node) => {
                nodes_id_map.insert(node.get_node_name().to_string(), node.get_node_id());
            }
        }
    }

    nodes_id_map
}

#[pymethods]
impl PyAsabrRouter {
    #[new]
    fn new(tvgutil_contact_plan_filepath: &str, router_type: &str) -> PyResult<Self> {
        let file = File::open(tvgutil_contact_plan_filepath).map_err(|e| {
            PyErr::new::<PyBaseException, _>(format!(
                "[A-SABR][File] Failed to open file '{}': {}",
                tvgutil_contact_plan_filepath, e
            ))
        })?;

        let json: serde_json::Value = serde_json::from_reader(file).map_err(|e| {
            PyErr::new::<PyBaseException, _>(format!(
                "[A-SABR][JSON] Failed to parse JSON: {}",
                e
            ))
        })?;

        make_guard!(id);

        // 1. Parse using SegmentationManager to match PyAsabrRouter field layout
        let contact_plan = TVGUtilContactPlan::parse::<NoManagement, SegmentationManager>(json)
            .map_err(|err| PyErr::new::<PyBaseException, _>(format!("[A-SABR] Parse error: {err}")))?;

        let nodes_id_map = make_nodes_id_map(&contact_plan.realnodes, &contact_plan.vnodes);

        let graph = Multigraph::new(id, contact_plan)
            .map_err(|err| PyErr::new::<PyBaseException, _>(format!("[A-SABR] Multigraph error: {err}")))?;

        // 2. Pass SegmentationManager to mk_router!
        // 2. Pass SegmentationManager to mk_router!
     let make_router_fn = || -> Result<_, ASABRError> {
            let r = mk_router!(
                id,
                NoManagement,
                SegmentationManager,
                1,
                router_type,
                graph
            )?;
            Ok(r)
        };
    let router = make_router_fn().map_err(|err| {
            PyValueError::new_err(format!("[A-SABR] Router creation error: {err:?}"))
        })?;

        // Erase lifetime bounds for PyO3 struct storage using transmute
        let router = unsafe { std::mem::transmute(router) };

        Ok(Self {
            nodes_id_map,
            router,
        })
    }


#[pyo3(name = "route")]
    fn route(
        &mut self,
        source: usize,
        bundle: PyAsabrBundle,
        curr_time: Date,
        _excluded_nodes: Vec<usize>,
    ) -> Vec<(PyAsabrContact, Vec<usize>)> {
        let native_bundle = bundle.to_native_bundle();

        let Ok(NodeRef::I(src)) = self.router.node_id_ref(NodeID::from(source)) else {
            return vec![];
        };

        if bundle.destinations.is_empty() {
            return vec![];
        }

        let Ok(isdest) = self.router.node_id_ref(NodeID::from(bundle.destinations[0])) else {
            return vec![];
        };

        let Ok(dest) = isdest.routable() else {
            return vec![];
        };

        if let Ok(Some((path_output, _first_hop))) = self.router.route(
            dest,
            curr_time,
            src,
            &native_bundle,
            None,
        ) {
            let mut py_routing_output = Vec::new();

            let contact = PyAsabrContact{
                    tx_node: src.into(),
                    rx_node: usize::from(self.router.into_nodeid(_first_hop.rx_node.into())),
                    start_time: _first_hop.via.unwrap().send.start,
                    end_time:  _first_hop.via.unwrap().send.end,
             };
             return py_routing_output;

        } else {
            Vec::new()
        }
    }

    #[pyo3(name = "get_node_id")]
    fn get_node_id(&self, node_name: &str) -> PyResult<usize> {
        if let Some(node_id) = self.nodes_id_map.get(node_name) {
            Ok(usize::from(*node_id))
        } else {
            Err(PyErr::new::<PyBaseException, _>(format!(
                "Node '{node_name}' unknown"
            )))
        }
    }
}