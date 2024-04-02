#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::env;
use std::str::FromStr;

use const_rollup_config::{ROLLUP_BATCH_NAMESPACE_RAW, ROLLUP_PROOF_NAMESPACE_RAW};
use sov_celestia_adapter::types::Namespace;
use sov_avail_adapter::service::*;
mod mock_rollup;
pub use mock_rollup::*;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

mod celestia_rollup;
pub use celestia_rollup::*;
#[cfg(feature = "experimental")]
mod eth;

mod avail_rollup;
pub use avail_rollup::*;

/// The rollup stores its data in the namespace b"sov-test" on Celestia
/// You can change this constant to point your rollup at a different namespace
pub const ROLLUP_BATCH_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_BATCH_NAMESPACE_RAW);

/// The rollup stores the zk proofs in the namespace b"sov-test-p" on Celestia.
pub const ROLLUP_PROOF_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_PROOF_NAMESPACE_RAW);

pub const LIGHT_CLIENT_URL: &str = "http://127.0.0.1:8000";

pub const NODE_CLIENT_URL: &str = "http://127.0.0.1:9944";

pub const SEED: &str = "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice";



/// Default initialization of logging
pub fn initialize_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::from_str(
                &env::var("RUST_LOG")
                    .unwrap_or_else(|_| "debug,hyper=info,risc0_zkvm=info".to_string()),
            )
            .unwrap(),
        )
        .init();
}
