use std::{collections::HashMap, fs::File};

use pyo3::{
    exceptions::{PyBaseException, PyValueError},
    prelude::*,
};

use a_sabr::{
    contact_manager::segmentation::seg::SegmentationManager, contact_plan::{RealNode, from_tvgutil_file::TVGUtilContactPlan}, distance::sabr::SABR, errors::ASABRError, mk_router, multigraph::{Multigraph, NodeRef, RoutableNodeRef}, node_manager::none::NoManagement, pathfinding::Pathfinding, types::{Date, NodeID}, utils::{Routing, make_guard}, vnode::VirtualNodeInfo,
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

/// Pathfinder trait object. In a type alias, `Box<dyn ...>` defaults to `+ 'static`.
type Pf = Box<
    dyn Pathfinding<'static, NoManagement, SegmentationManager, RoutableNodeRef<'static>>,
>;

/// Router trait object.
/// `Routing` has 4 generic parameters `<'id, NM, CM, D>`; the pathfinder is the
/// associated type `Pathfinder`, so it goes in `Pathfinder = ...`, not in the generics.
type DynRouter = Box<
    dyn Routing<
        'static,
        NoManagement,
        SegmentationManager,
        RoutableNodeRef<'static>, // D
        Pathfinder = Pf,
    >,
>;

#[pyclass(name = "AsabrRouter", unsendable)]
pub struct PyAsabrRouter {
    nodes_id_map: HashMap<String, NodeID>,
    router: DynRouter,
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
    fn new(
        tvgutil_contact_plan_filepath: &str,
        router_type: &str,
        args: Option<usize>,
    ) -> PyResult<Self> {
        let file = File::open(tvgutil_contact_plan_filepath).map_err(|e| {
            PyErr::new::<PyBaseException, _>(format!(
                "[A-SABR][File] Failed to open file '{}': {}",
                tvgutil_contact_plan_filepath, e
            ))
        })?;

        let json: serde_json::Value = serde_json::from_reader(file).map_err(|e| {
            PyErr::new::<PyBaseException, _>(format!("[A-SABR][JSON] Failed to parse JSON: {}", e))
        })?;

        make_guard!(id);

        let contact_plan = TVGUtilContactPlan::parse::<NoManagement, SegmentationManager>(json)
            .map_err(|err| {
                PyErr::new::<PyBaseException, _>(format!("[A-SABR] Parse error: {err}"))
            })?;

        let nodes_id_map = make_nodes_id_map(&contact_plan.realnodes, &contact_plan.vnodes);

        let graph = Multigraph::new(id, contact_plan).map_err(|err| {
            PyErr::new::<PyBaseException, _>(format!("[A-SABR] Multigraph error: {err}"))
        })?;

        // The macro uses `?` and `return Err(..)`, so it has to run inside a
        // function/closure that returns `Result<_, ASABRError>`.
        // NOTE: macro argument order must match your current macro definition:
        //   (id, router_type, NM, CM, prio_count, algo, multigraph, algo_args)
        let build_router = || -> Result<_, ASABRError> {
            mk_router!(
                id,
                NoManagement,
                SegmentationManager,
                1,
                router_type,
                graph,
                args,
                SABR
            )
        };

        let router = build_router().map_err(|err| {
            PyValueError::new_err(format!("[A-SABR] Router creation error: {err:?}"))
        })?;

        // SAFETY: only the generativity lifetime `'id` is changed to `'static`;
        // the layout is identical. The router owns its multigraph, and `INodeRef`s
        // are never handed out of this struct, so nothing can outlive it.
        // This does bypass the `'id` branding guarantee.
        let router: DynRouter = unsafe { std::mem::transmute(router) };

        Ok(Self {
            nodes_id_map,
            router,
        })
    }

    #[pyo3(name = "set_source")]
    fn set_source(&mut self, src: usize) -> PyResult<()> {
        // `node_id_ref` is a Multigraph method, reached through Deref on the router.
        let Ok(NodeRef::I(src_ref)) = self.router.node_id_ref(NodeID::from(src)) else {
            return Err(PyValueError::new_err("[A-SABR] Router source error"));
        };
        self.router.set_source(src_ref).into_py_res()
    }

    #[pyo3(name = "route")]
    fn route(
        &mut self,
        bundle: PyAsabrBundle,
        curr_time: Date,
        _excluded_nodes: Vec<usize>,
    ) -> Vec<(PyAsabrContact, Vec<usize>)> {
        if bundle.destinations.is_empty() {
            return vec![];
        }
        let native_bundle = bundle.to_native_bundle();
        let dest_id = bundle.destinations[0];

        let Ok(dest_ref) = self.router.node_id_ref(NodeID::from(dest_id)) else {
            return vec![];
        };
        let Ok(dest) = dest_ref.routable() else {
            return vec![];
        };

        // A single-source router stores its source as an `INodeRef`,
        // so bind it directly (no `NodeRef::I(..)` pattern).
        let Ok(src) = self.router.get_source() else {
            return vec![];
        };

        // `route` mutably borrows the router for as long as `first_hop` is alive,
        // so copy everything needed out of it before touching `self.router` again.
        // The `_` drops the path output right at the end of this statement.
        let Ok(Some((_, first_hop))) = self.router.route(dest, curr_time, &native_bundle, None)
        else {
            return vec![];
        };

        let Some(via) = first_hop.via else {
            return vec![];
        };
        let start_time = via.send.start;
        let end_time = via.send.end;
        let rx_ref = first_hop.rx_node.into();
        // `first_hop` is no longer used: the mutable borrow of the router ends here.

        let contact = PyAsabrContact {
            tx_node: usize::from(self.router.into_nodeid(src.into())),
            rx_node: usize::from(self.router.into_nodeid(rx_ref)),
            start_time,
            end_time,
        };

        // The original code never pushed anything, so it always returned an empty vec.
        // Second tuple element = destinations reached through this hop (assumption).
        vec![(contact, vec![usize::from(dest_id)])]
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