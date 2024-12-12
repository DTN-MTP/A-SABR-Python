use std::{cell::RefCell, rc::Rc};

use a_sabr::{
    bundle::Bundle,
    contact::Contact,
    contact_manager::seg::SegmentationManager,
    node::Node,
    node_manager::none::NoManagement,
    route_storage::cache::TreeCache,
    routing::{
        aliases::{SpsnHopMpt, SpsnHopNodeGraph, SpsnMpt, SpsnNodeGraph},
        spsn::Spsn,
        RoutingOutput,
    },
    types::{Date, NodeID},
};

#[cfg(feature = "contact_work_area")]
use a_sabr::routing::aliases::{SpsnContactGraph, SpsnHopContactGraph};

#[cfg(any(feature = "contact_suppression", feature = "first_depleted"))]
use a_sabr::{route_storage::table::RoutingTable, routing::cgr::Cgr};

#[cfg(feature = "contact_suppression")]
use a_sabr::routing::aliases::{
    CgrFirstEndingMpt, CgrFirstEndingNodeGraph, CgrHopFirstEndingMpt, CgrHopFirstEndingNodeGraph,
};

#[cfg(all(feature = "contact_suppression", feature = "contact_work_area"))]
use a_sabr::routing::aliases::{CgrFirstEndingContactGraph, CgrHopFirstEndingContactGraph};

use super::{DistanceStrategy, GenericRouter, PathfindingStrategy, RoutingFlavor};

pub fn make_generic_router(
    routing_flavor: RoutingFlavor,
    pathfinding_strategy: PathfindingStrategy,
    distance_strategy: DistanceStrategy,
    nodes: Vec<Node<NoManagement>>,
    contacts: Vec<Contact<SegmentationManager>>,
) -> Box<dyn GenericRouter<SegmentationManager>> {
    match routing_flavor {
        RoutingFlavor::SPSN => {
            let cache = Rc::new(RefCell::new(TreeCache::new(false, false, 10)));

            match (pathfinding_strategy, distance_strategy) {
                (PathfindingStrategy::NodeGraph, DistanceStrategy::Hop) => Box::new(
                    SpsnNodeGraphHopRouter(Spsn::new(nodes, contacts, cache, false)),
                ),
                (PathfindingStrategy::NodeGraph, DistanceStrategy::SABR) => Box::new(
                    SpsnNodeGraphSABRRouter(Spsn::new(nodes, contacts, cache, false)),
                ),
                (PathfindingStrategy::MultipathTracking, DistanceStrategy::Hop) => {
                    Box::new(SpsnMptHopRouter(Spsn::new(nodes, contacts, cache, false)))
                },
                (PathfindingStrategy::MultipathTracking, DistanceStrategy::SABR) => {
                    Box::new(SpsnMptSABRRouter(Spsn::new(nodes, contacts, cache, false)))
                },
                #[cfg(feature = "contact_work_area")]
                (PathfindingStrategy::ContactGraph, DistanceStrategy::Hop) =>
                    Box::new(
                    SpsnContactGraphHopRouter(Spsn::new(nodes, contacts, cache, false)),
                ),
                #[cfg(feature = "contact_work_area")]
                (PathfindingStrategy::ContactGraph, DistanceStrategy::SABR) => Box::new(
                    SpsnContactGraphSABRRouter(Spsn::new(nodes, contacts, cache, false)),
                ),
                #[cfg(not(feature = "contact_work_area"))]
                (PathfindingStrategy::ContactGraph, _) => panic!("Feature 'contact_work_area' must be enabled to use ContactGraph pathfinding with SPSN"),
            }
        }
        #[cfg(feature = "contact_suppression")]
        RoutingFlavor::CGRFirstEnding => {
            match (pathfinding_strategy, distance_strategy) {
                (PathfindingStrategy::NodeGraph, DistanceStrategy::Hop) => Box::new(
                    CgrHopFirstEndingNodeGraphRouter(Cgr::new(nodes, contacts, Rc::new(RefCell::new(RoutingTable::new())))),
                ),
                (PathfindingStrategy::NodeGraph, DistanceStrategy::SABR) => Box::new(
                    CgrSABRFirstEndingNodeGraphRouter(Cgr::new(nodes, contacts, Rc::new(RefCell::new(RoutingTable::new())))),
                ),
                (PathfindingStrategy::MultipathTracking, DistanceStrategy::Hop) => Box::new(
                    CgrHopFirstEndingMptRouter(Cgr::new(nodes, contacts, Rc::new(RefCell::new(RoutingTable::new())))),
                ),
                (PathfindingStrategy::MultipathTracking, DistanceStrategy::SABR) => Box::new(
                    CgrSABRFirstEndingMptRouter(Cgr::new(nodes, contacts, Rc::new(RefCell::new(RoutingTable::new())))),
                ),
                #[cfg(feature = "contact_work_area")]
                (PathfindingStrategy::ContactGraph, DistanceStrategy::Hop) => Box::new(
                    CgrHopFirstEndingContactGraphRouter(Cgr::new(nodes, contacts, Rc::new(RefCell::new(RoutingTable::new())))),
                ),
                #[cfg(feature = "contact_work_area")]
                (PathfindingStrategy::ContactGraph, DistanceStrategy::SABR) => Box::new(
                    CgrSABRFirstEndingContactGraphRouter(Cgr::new(nodes, contacts, Rc::new(RefCell::new(RoutingTable::new())))),
                ),
                #[cfg(not(feature = "contact_work_area"))]
                (PathfindingStrategy::ContactGraph, _) => panic!("Feature 'contact_work_area' must be enabled to use ContactGraph pathfinding with SPSN"),
            }
        }
        #[cfg(not(feature = "contact_suppression"))]
        RoutingFlavor::CGRFirstEnding => panic!(
            "Feature 'contact_suppression' must be enabled to use CGR FirstEnding pathfinding"
        ),
        RoutingFlavor::CGRFirstDepleted => {
            todo!();
        }
    }
}

macro_rules! generate_generic_router {
    ($router_name:ident, $routing_algo:ident, $node_manager:ident, $contact_manager:ident) => {
        struct $router_name($routing_algo<$node_manager, $contact_manager>);

        impl GenericRouter<$contact_manager> for $router_name {
            fn route(
                &mut self,
                source: NodeID,
                bundle: &Bundle,
                curr_time: Date,
                excluded_nodes: &Vec<NodeID>,
            ) -> Option<RoutingOutput<$contact_manager>> {
                self.0.route(source, bundle, curr_time, excluded_nodes)
            }
        }
    };
}

// SPSN routers
// ------------
generate_generic_router!(
    SpsnNodeGraphSABRRouter,
    SpsnNodeGraph,
    NoManagement,
    SegmentationManager
);
generate_generic_router!(
    SpsnNodeGraphHopRouter,
    SpsnHopNodeGraph,
    NoManagement,
    SegmentationManager
);
generate_generic_router!(
    SpsnMptSABRRouter,
    SpsnMpt,
    NoManagement,
    SegmentationManager
);
generate_generic_router!(
    SpsnMptHopRouter,
    SpsnHopMpt,
    NoManagement,
    SegmentationManager
);

// CGR routers
// ------------
#[cfg(feature = "contact_suppression")]
generate_generic_router!(
    CgrHopFirstEndingMptRouter,
    CgrHopFirstEndingMpt,
    NoManagement,
    SegmentationManager
);

#[cfg(feature = "contact_suppression")]
generate_generic_router!(
    CgrSABRFirstEndingMptRouter,
    CgrFirstEndingMpt,
    NoManagement,
    SegmentationManager
);
#[cfg(feature = "contact_suppression")]
generate_generic_router!(
    CgrHopFirstEndingNodeGraphRouter,
    CgrHopFirstEndingNodeGraph,
    NoManagement,
    SegmentationManager
);
#[cfg(feature = "contact_suppression")]
generate_generic_router!(
    CgrSABRFirstEndingNodeGraphRouter,
    CgrFirstEndingNodeGraph,
    NoManagement,
    SegmentationManager
);

#[cfg(all(feature = "contact_work_area", feature = "contact_suppression"))]
generate_generic_router!(
    CgrHopFirstEndingContactGraphRouter,
    CgrHopFirstEndingContactGraph,
    NoManagement,
    SegmentationManager
);
#[cfg(all(feature = "contact_work_area", feature = "contact_suppression"))]
generate_generic_router!(
    CgrSABRFirstEndingContactGraphRouter,
    CgrFirstEndingContactGraph,
    NoManagement,
    SegmentationManager
);

// Routers from "contact_work_area" feature
// ----------------------------------------
#[cfg(feature = "contact_work_area")]
generate_generic_router!(
    SpsnContactGraphSABRRouter,
    SpsnContactGraph,
    NoManagement,
    SegmentationManager
);

#[cfg(feature = "contact_work_area")]
generate_generic_router!(
    SpsnContactGraphHopRouter,
    SpsnHopContactGraph,
    NoManagement,
    SegmentationManager
);
