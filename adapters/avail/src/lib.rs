
#[cfg(feature = "native")]
mod avail;
pub mod service;
pub mod spec;
pub mod verifier;

// NOTE: Remove once dependency to the node is removed
#[cfg(feature = "native")]
pub use service::{DaServiceConfig, DaProvider};
pub use avail_subxt::build_client;
pub use verifier::Verifier;
