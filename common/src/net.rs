

#[derive(Debug, Clone, Copy, strum::Display)]
pub enum ConnectionType {
    Plain,
    Ssl,
    Auto, // AutoDetect
}
