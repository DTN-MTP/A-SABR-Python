pub enum DistanceStrategy {
    Hop,
    SABR,
}

impl DistanceStrategy {
    pub fn from_str(distance_strategy_name: &str) -> Option<Self> {
        match distance_strategy_name.to_lowercase().as_str() {
            "hop" => Some(DistanceStrategy::Hop),
            "sabr" => Some(DistanceStrategy::SABR),
            _ => None,
        }
    }
}
