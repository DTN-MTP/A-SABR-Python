pub enum RoutingFlavor {
    SPSN,
    CGR,
}

impl RoutingFlavor {
    pub fn from_str(routing_flavor_name: &str) -> Option<Self> {
        match routing_flavor_name.to_lowercase().as_str() {
            "spsn" => Some(RoutingFlavor::SPSN),
            "cgr" => Some(RoutingFlavor::CGR),
            _ => None,
        }
    }
}
