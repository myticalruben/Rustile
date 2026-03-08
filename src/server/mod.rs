use crate::RustileConfig;

pub struct RustileServer{
    pub config: RustileConfig
}

impl RustileServer {
    pub fn new(config: RustileConfig) -> Self{
        Self { config }
    }    

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>>{
        println!("🚀 Iniciando Rustile Wayland Compositor...");
        Ok(())
    }
}