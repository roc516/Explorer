mod path;
mod session;
mod fsbackend;

pub struct ZipBackend;
pub const ID: &str = "zip";
pub const EXTENSIONS: &[&str] = &["zip", "jar", "apk"];
