pub enum PathfindingStrategy {
    NodeGraph,
    ContactGraph,
    MultipathTracking,
}

impl PathfindingStrategy {
    pub fn from_str(pathfinding_strategy_name: &str) -> Option<Self> {
        match pathfinding_strategy_name.to_lowercase().as_str() {
            "node_graph" => Some(PathfindingStrategy::NodeGraph),
            "contact_graph" => Some(PathfindingStrategy::ContactGraph),
            "multipath_tracking" => Some(PathfindingStrategy::MultipathTracking),
            _ => None,
        }
    }
}
