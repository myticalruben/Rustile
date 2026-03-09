use crate::RustileConfig;

pub struct RustileState{
    pub config: RustileConfig,
    pub is_running: bool
}

impl RustileState{
    pub fn new(config: RustileConfig) -> Self{
        Self {
            config,
            is_running: true
        }
    }
}