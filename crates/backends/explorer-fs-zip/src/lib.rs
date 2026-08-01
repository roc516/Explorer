mod archive;
mod dir;
mod file;
mod fsbackend;
mod path;
mod reader;

pub struct ZipBackend;
pub const ID: &str = "zip";
pub const EXTENSIONS: &[&str] = &["zip", "jar", "apk"];
