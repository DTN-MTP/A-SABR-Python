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

#[cfg(feature = "first_depleted")]
use a_sabr::routing::aliases::{
    CgrFirstDepletedMpt, CgrFirstDepletedNodeGraph, CgrHopFirstDepletedMpt,
    CgrHopFirstDepletedNodeGraph,
};

#[cfg(all(feature = "first_depleted", feature = "contact_work_area"))]
use a_sabr::routing::aliases::{CgrFirstDepletedContactGraph, CgrHopFirstDepletedContactGraph};

use super::GenericRouter;

macro_rules! register_spsn_router {
    ($router:ident, $router_name:literal, $test_name_variable:ident, $nodes:ident, $contacts:ident) => {
        if $test_name_variable == $router_name {
            let cache = Rc::new(RefCell::new(TreeCache::new(false, false, 10)));

            return Box::new($router(Spsn::new($nodes, $contacts, cache, false)));
        }
    };
}

macro_rules! register_cgr_router {
    ($router:ident, $router_name:literal, $test_name_variable:ident, $nodes:ident, $contacts:ident) => {
        if $test_name_variable == $router_name {
            let routing_table = Rc::new(RefCell::new(RoutingTable::new()));

            return Box::new($router(Cgr::new($nodes, $contacts, routing_table)));
        }
    };
}

#[rustfmt::skip]
pub fn make_generic_router(
    router_type: &str,
    nodes: Vec<Node<NoManagement>>,
    contacts: Vec<Contact<SegmentationManager>>,
) -> Box<dyn GenericRouter<SegmentationManager>> {
    register_spsn_router!(SpsnNodeGraphRouter, "SpsnNodeGraph", router_type, nodes, contacts);
    register_spsn_router!(SpsnHopNodeGraphRouter, "SpsnHopNodeGraph", router_type, nodes, contacts);
    register_spsn_router!(SpsnMptRouter, "SpsnMpt", router_type, nodes, contacts);
    register_spsn_router!(SpsnHopMptRouter, "SpsnHopMpt", router_type, nodes, contacts);

    #[cfg(feature = "contact_work_area")]
    {
        register_spsn_router!(SpsnContactGraphRouter, "SpsnContactGraph", router_type, nodes, contacts);
        register_spsn_router!(SpsnHopContactGraphRouter, "SpsnHopContactGraph", router_type, nodes, contacts);
    }

    #[cfg(feature = "contact_suppression")]
    {
        register_cgr_router!(CgrHopFirstEndingMptRouter, "CgrHopFirstEndingMpt", router_type, nodes, contacts);
        register_cgr_router!(CgrFirstEndingMptRouter, "CgrFirstEndingMpt", router_type, nodes, contacts);
        register_cgr_router!(CgrHopFirstEndingNodeGraphRouter, "CgrHopFirstEndingNodeGraph", router_type, nodes, contacts);
        register_cgr_router!(CgrFirstEndingNodeGraphRouter, "CgrFirstEndingNodeGraph", router_type, nodes, contacts);

        #[cfg(feature = "contact_work_area")]
        {
            register_cgr_router!(CgrHopFirstEndingContactGraphRouter, "CgrHopFirstEndingContactGraph", router_type, nodes, contacts);
            register_cgr_router!(CgrFirstEndingContactGraphRouter, "CgrFirstEndingContactGraph", router_type, nodes, contacts);
        }
    }

    #[cfg(feature = "first_depleted")]
    {
        register_cgr_router!(CgrHopFirstDepletedMptRouter, "CgrHopFirstDepletedMpt", router_type, nodes, contacts);
        register_cgr_router!(CgrFirstDepletedMptRouter, "CgrFirstDepletedMpt", router_type, nodes, contacts);
        register_cgr_router!(CgrHopFirstDepletedNodeGraphRouter, "CgrHopFirstDepletedNodeGraph", router_type, nodes, contacts);
        register_cgr_router!(CgrFirstDepletedNodeGraphRouter, "CgrFirstDepletedNodeGraph", router_type, nodes, contacts);

        #[cfg(feature = "contact_work_area")]
        {
            register_cgr_router!(CgrHopFirstDepletedContactGraphRouter, "CgrHopFirstDepletedContactGraph", router_type, nodes, contacts);
            register_cgr_router!(CgrFirstDepletedContactGraphRouter, "CgrFirstDepletedContactGraph", router_type, nodes, contacts);
        }
    }

    panic!(
        "Router type \"{}\" is invalid! (check for typo or disabled feature)",
        &router_type
    );
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
    SpsnNodeGraphRouter,
    SpsnNodeGraph,
    NoManagement,
    SegmentationManager
);
generate_generic_router!(
    SpsnHopNodeGraphRouter,
    SpsnHopNodeGraph,
    NoManagement,
    SegmentationManager
);
generate_generic_router!(SpsnMptRouter, SpsnMpt, NoManagement, SegmentationManager);
generate_generic_router!(
    SpsnHopMptRouter,
    SpsnHopMpt,
    NoManagement,
    SegmentationManager
);

#[cfg(feature = "contact_work_area")]
generate_generic_router!(
    SpsnContactGraphRouter,
    SpsnContactGraph,
    NoManagement,
    SegmentationManager
);

#[cfg(feature = "contact_work_area")]
generate_generic_router!(
    SpsnHopContactGraphRouter,
    SpsnHopContactGraph,
    NoManagement,
    SegmentationManager
);

// CGR routers [FirstEnding flavor]
// --------------------------------
#[cfg(feature = "contact_suppression")]
generate_generic_router!(
    CgrHopFirstEndingMptRouter,
    CgrHopFirstEndingMpt,
    NoManagement,
    SegmentationManager
);
#[cfg(feature = "contact_suppression")]
generate_generic_router!(
    CgrFirstEndingMptRouter,
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
    CgrFirstEndingNodeGraphRouter,
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
    CgrFirstEndingContactGraphRouter,
    CgrFirstEndingContactGraph,
    NoManagement,
    SegmentationManager
);

// CGR routers [FirstDepleted flavor]
// --------------------------------
#[cfg(feature = "first_depleted")]
generate_generic_router!(
    CgrHopFirstDepletedMptRouter,
    CgrHopFirstDepletedMpt,
    NoManagement,
    SegmentationManager
);
#[cfg(feature = "first_depleted")]
generate_generic_router!(
    CgrFirstDepletedMptRouter,
    CgrFirstDepletedMpt,
    NoManagement,
    SegmentationManager
);
#[cfg(feature = "first_depleted")]
generate_generic_router!(
    CgrHopFirstDepletedNodeGraphRouter,
    CgrHopFirstDepletedNodeGraph,
    NoManagement,
    SegmentationManager
);
#[cfg(feature = "first_depleted")]
generate_generic_router!(
    CgrFirstDepletedNodeGraphRouter,
    CgrFirstDepletedNodeGraph,
    NoManagement,
    SegmentationManager
);

#[cfg(all(feature = "contact_work_area", feature = "first_depleted"))]
generate_generic_router!(
    CgrHopFirstDepletedContactGraphRouter,
    CgrHopFirstDepletedContactGraph,
    NoManagement,
    SegmentationManager
);
#[cfg(all(feature = "contact_work_area", feature = "first_depleted"))]
generate_generic_router!(
    CgrFirstDepletedContactGraphRouter,
    CgrFirstDepletedContactGraph,
    NoManagement,
    SegmentationManager
);
