#[derive(Debug, Clone)]
pub struct RustileConfig {
    pub border_width: u32,
    pub color_focus: u32,
    pub color_normal: u32,
    pub gap_size: u32,
    pub workspaces: Vec<String>,
}

impl Default for RustileConfig {
    fn default() -> Self {
        Self {
            border_width: 2,
            color_focus: 0xffaa00,
            color_normal: 0x444444,
            gap_size: 5,
            workspaces: vec!["1".into(),"2".into(),"3".into(),"4".into()],
        }
    }
}
