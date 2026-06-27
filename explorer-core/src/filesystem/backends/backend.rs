use super::{ArchiveMount, BackendBootstrap, BackendIdentity, FsIo, PathMetadata};

pub trait FsBackend:
    BackendIdentity
    + BackendBootstrap
    + PathMetadata
    + FsIo
    + ArchiveMount
    + Send
    + Sync
{
}
