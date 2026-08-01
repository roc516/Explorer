use std::path::Path;

use explorer_core::BlockDevice;
use explorer_core::filesystem::{FsBackend, MountedFs};

use crate::archive::ZipFs;

fn extension_matches(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| crate::EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn looks_like_zip(device: &BlockDevice) -> bool {
    let mut magic = [0u8; 4];
    match device.read_at(0, &mut magic) {
        Ok(n) if n >= 2 => {}
        _ => return false,
    }
    matches!(&magic[..2], b"PK")
        && (magic[2] == 0x03 || magic[2] == 0x05 || magic[2] == 0x07)
}

impl FsBackend for crate::ZipBackend {
    fn id(&self) -> &'static str {
        crate::ID
    }

    fn matches(&self, device: &BlockDevice) -> bool {
        if extension_matches(device.name()) {
            return true;
        }
        looks_like_zip(device)
    }

    fn mount(&self, device: &BlockDevice) -> Result<Box<dyn MountedFs>, String> {
        ZipFs::open(device).map(|fs| Box::new(fs) as Box<dyn MountedFs>)
    }
}
