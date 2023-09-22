#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod call;
pub use call::CallMessage;
mod address;
mod genesis;
pub use address::*;
mod collection;
use collection::*;
mod nft;
use nft::*;
#[cfg(feature = "native")]
mod query;
#[cfg(feature = "native")]
pub use query::*;
use sov_modules_api::{CallResponse, Context, Error, Module, ModuleInfo, StateMap, WorkingSet};
/// Utility functions.
pub mod utils;

#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
#[derive(ModuleInfo, Clone)]
/// Module for non-fungible tokens (NFT).
/// Each token is represented by a unique ID.
pub struct NonFungibleToken<C: Context> {
    #[address]
    /// The address of the NonFungibleToken module.
    address: C::Address,

    #[state]
    /// Mapping of tokens to their owners
    collections: StateMap<CollectionAddress<C>, Collection<C>>,

    #[state]
    /// Mapping of tokens to their owners
    nfts: StateMap<NftIdentifier<C>, Nft<C>>,
}

/// Config for the NonFungibleToken module.
/// Sets admin and existing owners.
pub struct NonFungibleTokenConfig {}

impl<C: Context> Module for NonFungibleToken<C> {
    type Context = C;

    type Config = NonFungibleTokenConfig;

    type CallMessage = CallMessage<C>;

    fn genesis(
        &self,
        _config: &Self::Config,
        _working_set: &mut WorkingSet<C>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Self::Context,
        working_set: &mut WorkingSet<C>,
    ) -> Result<CallResponse, Error> {
        let call_result = match msg {
            CallMessage::CreateCollection {
                name,
                collection_uri,
                authorized_minters,
            } => self.create_collection(&name, &collection_uri, authorized_minters, context, working_set),
            CallMessage::FreezeCollection { collection_address } => {
                self.freeze_collection(&collection_address, context, working_set)
            }
            CallMessage::MintNft {
                collection_address,
                token_uri,
                token_id,
                owner,
                frozen,
            } => self.mint_nft_from_eoa(
                token_id,
                &collection_address,
                &token_uri,
                &owner,
                frozen,
                context,
                working_set,
            ),
            CallMessage::UpdateCollection {
                collection_address,
                collection_uri,
            } => self.update_collection(&collection_address, &collection_uri, context, working_set),
            CallMessage::TransferNft {
                collection_address,
                token_id,
                to,
            } => self.transfer_nft(token_id, &collection_address, &to, context, working_set),
            CallMessage::UpdateNft {
                collection_address,
                token_id,
                token_uri,
                frozen,
            } => self.update_nft(
                &collection_address,
                token_id,
                token_uri,
                frozen,
                context,
                working_set,
            ),
        };
        Ok(call_result?)
    }
}
