/// The namespace used by the rollup to store its data. This is a raw slice of 8 bytes.
/// The rollup stores its data in the namespace b"sov-test" on Celestia. Which in this case is encoded using the
/// ascii representation of each character.
pub const ROLLUP_BATCH_NAMESPACE_RAW: [u8; 10] = [0, 0, 115, 111, 118, 45, 116, 101, 115, 116];

/// The namespace used by the rollup to store aggregated ZK proofs.
pub const ROLLUP_PROOF_NAMESPACE_RAW: [u8; 10] = [115, 111, 118, 45, 116, 101, 115, 116, 45, 112];

/// light client url
pub const LIGHT_CLIENT_URL: &str = "http://127.0.0.1:8000";

/// node client url 
pub const NODE_CLIENT_URL: &str = "ws://127.0.0.1:9944";

/// default polling interval
pub const DEFAULT_POLLING_INTERVAL: [u64; 1] = [30];

/// default polling timeout
pub const DEFAULT_POLLING_TIMEOUT: [u64; 1] = [5];

/// seed phrase..
pub const SEED: &str = "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice";


