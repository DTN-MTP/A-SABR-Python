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
        RoutingFlavor::CGR => {
            todo!();
        }
    }
}

macro_rules! generate_generic_router {
    ($router_wrapper:ident, $contact_manager_type:ident) => {
        impl GenericRouter<$contact_manager_type> for $router_wrapper {
            fn route(
                &mut self,
                source: NodeID,
                bundle: &Bundle,
                curr_time: Date,
                excluded_nodes: &Vec<NodeID>,
            ) -> Option<RoutingOutput<$contact_manager_type>> {
                self.0.route(source, bundle, curr_time, excluded_nodes)
            }
        }
    };
}

// SPSN routers
// ------------
struct SpsnNodeGraphSABRRouter(SpsnNodeGraph<NoManagement, SegmentationManager>);
generate_generic_router!(SpsnNodeGraphSABRRouter, SegmentationManager);

struct SpsnNodeGraphHopRouter(SpsnHopNodeGraph<NoManagement, SegmentationManager>);
generate_generic_router!(SpsnNodeGraphHopRouter, SegmentationManager);

struct SpsnMptSABRRouter(SpsnMpt<NoManagement, SegmentationManager>);
generate_generic_router!(SpsnMptSABRRouter, SegmentationManager);

struct SpsnMptHopRouter(SpsnHopMpt<NoManagement, SegmentationManager>);
generate_generic_router!(SpsnMptHopRouter, SegmentationManager);

// CGR routers
// ------------
// TODO

// Routers from "contact_work_area" feature
// ----------------------------------------
#[cfg(feature = "contact_work_area")]
struct SpsnContactGraphSABRRouter(SpsnContactGraph<NoManagement, SegmentationManager>);
#[cfg(feature = "contact_work_area")]
generate_generic_router!(SpsnContactGraphSABRRouter, SegmentationManager);

#[cfg(feature = "contact_work_area")]
struct SpsnContactGraphHopRouter(SpsnHopContactGraph<NoManagement, SegmentationManager>);
#[cfg(feature = "contact_work_area")]
generate_generic_router!(SpsnContactGraphHopRouter, SegmentationManager);
