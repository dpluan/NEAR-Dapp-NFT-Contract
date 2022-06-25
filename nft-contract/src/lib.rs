use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{collections::LookupMap, AccountId};
use near_sdk::{
    env, ext_contract, log, near_bindgen, Balance, CryptoHash, Gas, PanicOnDefault, Promise,
    PromiseOrValue, PromiseResult,
};
use std::collections::HashMap;

pub use crate::approval::*;
pub use crate::enumeration::*;
pub use crate::event::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::royalty::*;
pub use crate::utils::*;
mod approval;
mod enumeration;
mod event;
mod internal;
mod metadata;
mod mint;
mod nft_core;
mod royalty;
mod utils;

pub type TokenId = String;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
struct Contract {
    pub owner_id: AccountId,
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub tokens_by_id: LookupMap<TokenId, Token>,
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,
    pub metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum StorageKey {
    TokenPerOwnerKey,
    ContractMetadataKey,
    TokenByIdKey,
    TokenMetaDataByIdKey,
    TokenPerOwnerInnerKey { account_id_hash: CryptoHash },
    UsersPerTokenKey { token_id_hash: CryptoHash },
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, token_metadata: NFTContractMetadata) -> Self {
        Self {
            owner_id: owner_id,
            tokens_per_owner: LookupMap::new(StorageKey::TokenPerOwnerKey.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokenByIdKey.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetaDataByIdKey.try_to_vec().unwrap(),
            ),
            metadata: LazyOption::new(
                StorageKey::TokenPerOwnerKey.try_to_vec().unwrap(),
                Some(&token_metadata),
            ),
        }
    }

    #[init]
    pub fn new_default_metadata(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "sc-nft-0.0.1".to_string(),
                name: "Smart Contract NFT".to_string(),
                symbol: "SCNFT".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;
    const MINT_STORAGE_COST: u128 = 58700000000000000000000;

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .is_view(is_view);
        builder
    }

    fn get_sample_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("TOKEN_TEST".to_owned()),
            description: Some("Description".to_owned()),
            media: None,
            media_hash: None,
            copies: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_mint_token() {
        let mut context = get_context(false);
        testing_env!(context.build());
        let mut contract = Contract::new_default_metadata(env::predecessor_account_id());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "SC_1".to_string();
        contract.nft_mint(
            token_id.clone(),
            get_sample_metadata(),
            accounts(0).to_string(),
        );

        let token = contract.nft_token(token_id.clone()).unwrap();

        assert_eq!(accounts(0).to_string(), token.owner_id);
        assert_eq!(token_id.clone(), token.token_id);
        assert_eq!(token.metadata, get_sample_metadata());
    }
}
