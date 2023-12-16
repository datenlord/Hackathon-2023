/// Default configuration file.
pub const DEFAULT_CONFIG_FILE: &str = "config.toml";
/// Default configuration file content.
pub static DEFAULT_CONFIG_CONTENT: &[u8] = include_bytes!("../config.toml");
/// Default mode
pub const DEFAULT_SERVER_MODE: &str = "tracker"; // "tracker" or "node"
pub const SERVER_MODE_TRACKER: &str = "tracker"; // Tracker mode
pub const SERVER_MODE_NODE: &str = "node"; // Node mode
/// Default filename
pub const DEFAULT_FILENAME: &str = "output_10MB.txt";
