#[derive(Debug, Clone)]
pub struct Monitor {
    pub display_number: u8,
    #[allow(dead_code)]
    pub i2c_bus: Option<String>,
    pub name: String,
}
