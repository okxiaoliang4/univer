#[cfg(feature = "server")]
pub mod document;
#[cfg(feature = "server")]
pub mod ot;
#[cfg(feature = "server")]
pub mod snapshot;

#[cfg(feature = "server")]
pub use document::DocumentService;
#[cfg(feature = "server")]
pub use ot::OTService;
#[cfg(feature = "server")]
pub use snapshot::SnapshotService;
