mod path;
mod fsbackend;

pub struct ZipBackend;
pub const ID: &str = "zip";
pub const EXTENSIONS: &[&str] = &["zip", "jar", "apk"];
