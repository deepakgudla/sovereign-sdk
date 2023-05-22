//! This module defines the following tables:
//!
//! Slot Tables:
//! - SlotNumber -> StoredSlot
//! - SlotNumber -> Vec<BatchNumber>
//!
//! Batch Tables:
//! - BatchNumber -> StoredBatch
//! - BatchHash -> BatchNumber
//!
//! Tx Tables:
//! - TxNumber -> (TxHash,Tx)
//! - TxHash -> TxNumber
//!
//! Event Tables:
//! - (EventKey, TxNumber) -> EventNumber
//! - EventNumber -> (EventKey, EventValue)

use super::types::{
    BatchNumber, DbHash, EventNumber, JmtValue, SlotNumber, StateKey, StoredBatch, StoredSlot,
    StoredTransaction, TxNumber,
};

use borsh::{maybestd, BorshDeserialize, BorshSerialize};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use jmt::{
    storage::{Node, NodeKey},
    Version,
};
use sovereign_core::db::errors::CodecError;
use sovereign_core::{
    db::{KeyDecoder, KeyEncoder, ValueCodec},
    stf::{Event, EventKey},
};

pub const STATE_TABLES: &[&str] = &[
    KeyHashToKey::table_name(),
    JmtValues::table_name(),
    JmtNodes::table_name(),
];

pub const LEDGER_TABLES: &[&str] = &[
    SlotByNumber::table_name(),
    SlotByHash::table_name(),
    BatchByHash::table_name(),
    BatchByNumber::table_name(),
    TxByHash::table_name(),
    TxByNumber::table_name(),
    EventByKey::table_name(),
    EventByNumber::table_name(),
];

/// Macro to define a table that implements [`sovereign_core::db::Schema`].
/// KeyCodec<Schema> and ValueCodec<Schema> must be implemented separately.
///
/// ```ignore
/// define_table_without_codec!(
///  /// A table storing keys and value
///  (MyTable) MyKey | MyValue
/// )
///
/// // This impl must be written by hand
/// impl KeyCodec<MyTable> for MyKey {
/// // ...
/// }
///
/// // This impl must be written by hand
/// impl ValueCodec<MyTable> for MyValue {
/// // ...
/// }
/// ```
macro_rules! define_table_without_codec {
    ($(#[$docs:meta])+ ( $table_name:ident ) $key:ty => $value:ty) => {
        $(#[$docs])+
        ///
        #[doc = concat!("Takes [`", stringify!($key), "`] as a key and returns [`", stringify!($value), "`]")]
        #[derive(Clone, Copy, Debug, Default)]
        pub(crate) struct $table_name;

        impl ::sovereign_core::db::Schema for $table_name {
            const COLUMN_FAMILY_NAME: &'static str = $table_name::table_name();
            type Key = $key;
            type Value = $value;
        }

        impl $table_name {
            #[doc=concat!("Return ", stringify!($table_name), " as it is present inside the database.")]
            pub const fn table_name() -> &'static str {
                ::core::stringify!($table_name)
            }
        }

        impl ::std::fmt::Display for $table_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::core::write!(f, "{}", stringify!($table_name))
            }
        }
    };
}

/// Macro to define a table that implements [`sovereign_core::db::Schema`].
/// Automatically generates KeyCodec<...> and ValueCodec<...> implementations
/// using the Encode and Decode traits from sovereign_core
///
/// ```ignore
/// define_table_with_default_codec!(
///  /// A table storing keys and value
///  (MyTable) MyKey | MyValue
/// )
/// ```
macro_rules! define_table_with_default_codec {
    ($(#[$docs:meta])+ ($table_name:ident) $key:ty => $value:ty) => {
        define_table_without_codec!($(#[$docs])+ ( $table_name ) $key => $value);

        impl ::sovereign_core::db::KeyEncoder<$table_name> for $key {
            fn encode_key(&self) -> ::std::result::Result<::sovereign_core::maybestd::vec::Vec<u8>, ::sovereign_core::db::errors::CodecError> {
                <Self as ::borsh::BorshSerialize>::try_to_vec(self).map_err(|e| e.into())
            }
        }

        impl ::sovereign_core::db::KeyDecoder<$table_name> for $key {
            fn decode_key(data: &[u8]) -> ::std::result::Result<Self, ::sovereign_core::db::errors::CodecError> {
                <Self as ::borsh::BorshDeserialize>::deserialize_reader(&mut &data[..]).map_err(|e| e.into())
            }
        }

        impl ::sovereign_core::db::ValueCodec<$table_name> for $value {
            fn encode_value(&self) -> ::std::result::Result<::sovereign_core::maybestd::vec::Vec<u8>, ::sovereign_core::db::errors::CodecError> {
                <Self as ::borsh::BorshSerialize>::try_to_vec(self).map_err(|e| e.into())
            }

            fn decode_value(data: &[u8]) -> ::std::result::Result<Self, ::sovereign_core::db::errors::CodecError> {
                <Self as ::borsh::BorshDeserialize>::deserialize_reader(&mut &data[..]).map_err(|e| e.into())
            }
        }
    };
}

// fn deser(target: &mut &[u8]) -> Result<Self, DeserializationError>;
define_table_with_default_codec!(
    /// The primary source for slot data
    (SlotByNumber) SlotNumber => StoredSlot
);

define_table_with_default_codec!(
    /// A "secondary index" for slot data by hash
    (SlotByHash) DbHash => SlotNumber
);

define_table_with_default_codec!(
    /// The primary source for batch data
    (BatchByNumber) BatchNumber => StoredBatch
);

define_table_with_default_codec!(
    /// A "secondary index" for batch data by hash
    (BatchByHash) DbHash => BatchNumber
);

define_table_with_default_codec!(
    /// The primary source for transaction data
    (TxByNumber) TxNumber => StoredTransaction
);

define_table_with_default_codec!(
    /// A "secondary index" for transaction data by hash
    (TxByHash) DbHash => TxNumber
);

define_table_with_default_codec!(
    /// The primary store for event data
    (EventByNumber) EventNumber => Event
);

define_table_with_default_codec!(
    /// A "secondary index" for event data by key
    (EventByKey) (EventKey, TxNumber, EventNumber) => ()
);

define_table_without_codec!(
    /// The source of truth for JMT nodes
    (JmtNodes) NodeKey => Node
);

impl KeyEncoder<JmtNodes> for NodeKey {
    fn encode_key(&self) -> sovereign_core::db::Result<Vec<u8>> {
        self.try_to_vec().map_err(CodecError::from)
    }
}
impl KeyDecoder<JmtNodes> for NodeKey {
    fn decode_key(data: &[u8]) -> sovereign_core::db::Result<Self> {
        Ok(Self::deserialize_reader(&mut &data[..])?)
    }
}

impl ValueCodec<JmtNodes> for Node {
    fn encode_value(&self) -> sovereign_core::db::Result<Vec<u8>> {
        self.try_to_vec().map_err(CodecError::from)
    }

    fn decode_value(data: &[u8]) -> sovereign_core::db::Result<Self> {
        Ok(Self::deserialize_reader(&mut &data[..])?)
    }
}

define_table_without_codec!(
    /// The source of truth for JMT values by version
    (JmtValues) (StateKey, Version) => JmtValue
);

impl<T: AsRef<[u8]> + PartialEq + core::fmt::Debug> KeyEncoder<JmtValues> for (T, Version) {
    fn encode_key(&self) -> sovereign_core::db::Result<Vec<u8>> {
        let mut out =
            Vec::with_capacity(self.0.as_ref().len() + std::mem::size_of::<Version>() + 8);
        self.0
            .as_ref()
            .serialize(&mut out)
            .map_err(CodecError::from)?;
        // Write the version in big-endian order so that sorting order is based on the most-significant bytes of the key
        out.write_u64::<BigEndian>(self.1)
            .expect("serialization to vec is infallible");
        Ok(out)
    }
}

impl KeyDecoder<JmtValues> for (StateKey, Version) {
    fn decode_key(data: &[u8]) -> sovereign_core::db::Result<Self> {
        let mut cursor = maybestd::io::Cursor::new(data);
        let key = Vec::<u8>::deserialize_reader(&mut cursor)?;
        let version = cursor.read_u64::<BigEndian>()?;
        Ok((key, version))
    }
}

impl ValueCodec<JmtValues> for JmtValue {
    fn encode_value(&self) -> sovereign_core::db::Result<Vec<u8>> {
        self.try_to_vec().map_err(CodecError::from)
    }

    fn decode_value(data: &[u8]) -> sovereign_core::db::Result<Self> {
        Ok(Self::deserialize_reader(&mut &data[..])?)
    }
}

define_table_with_default_codec!(
    /// A mapping from key-hashes to their preimages and latest version. Since we store raw
    /// key-value pairs instead of keyHash->value pairs,
    /// this table is required to implement the `jmt::TreeReader` trait,
    /// which requires the ability to fetch values by hash.
    (KeyHashToKey) [u8;32] => StateKey
);