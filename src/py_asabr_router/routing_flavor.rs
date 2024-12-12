pub enum RoutingFlavor {
    SPSN,
    CGRFirstEnding,
    CGRFirstDepleted,
}

impl RoutingFlavor {
    pub fn from_str(routing_flavor_name: &str) -> Option<Self> {
        match routing_flavor_name.to_lowercase().as_str() {
            "spsn" => Some(RoutingFlavor::SPSN),
            "cgr_firstending" => Some(RoutingFlavor::CGRFirstEnding),
            "cgr_firstdepleted" => Some(RoutingFlavor::CGRFirstDepleted),
            _ => None,
        }
    }
}
