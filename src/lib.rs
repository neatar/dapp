use std::convert::TryFrom;
use std::convert::TryInto;

use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{hash_account_id, NonFungibleToken};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{Base64VecU8, ValidAccountId};
use near_sdk::serde_json::json;
use near_sdk::Balance;
use near_sdk::Gas;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

mod identicon;

// Prepaid gas for making a single simple call.
const SINGLE_CALL_GAS: Gas = 200000000000000;
const ONE_YOCTO: Balance = 1;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

const RAW: u64 = 0x55;
const DEFAULT_AVATAR: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let contract_id = env::current_account_id();
        let owner_id = ValidAccountId::try_from(contract_id.clone()).unwrap();
        Self {
            token: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(
                StorageKey::Metadata,
                Some(&NFTContractMetadata {
                    spec: NFT_METADATA_SPEC.to_string(),
                    name: "Avatar for Web3".to_string(),
                    symbol: "AVATAR".to_string(),
                    icon: Some(identicon::make(contract_id.as_bytes())),
                    base_uri: Some("data:image".to_string()),
                    reference: None,
                    reference_hash: None,
                }),
            ),
        }
    }

    pub fn avatar_of(self, account_id: ValidAccountId) -> String {
        let list = self.token.nft_tokens_for_owner(account_id, None, None);
        if list.is_empty() {
            return DEFAULT_AVATAR.to_string();
        }
        let media = list
            .last()
            .cloned()
            .unwrap()
            .metadata
            .unwrap()
            .media
            .unwrap();
        format!("data:image/{}", media)
    }

    #[payable]
    pub fn avatar_create(&mut self) -> String {
        let owner_id = env::signer_account_id().try_into().unwrap();
        self.avatar_create_for(owner_id)
    }

    #[payable]
    pub fn avatar_create_for(&mut self, owner_id: ValidAccountId) -> String {
        let svg = identicon::make(&hash_account_id(&owner_id.clone().into()));
        let media = format!("svg+xml;base64,{}", base64::encode(svg.clone()));
        let hash = Code::Sha2_256.digest(svg.as_bytes());
        let media_hash = Base64VecU8(env::sha256(svg.as_bytes()));
        let token_id = Cid::new_v1(RAW, hash).to_string();
        self.token.owner_id = env::signer_account_id(); // FIXME
        let current_account_id = env::current_account_id();
        let escrow_id = ValidAccountId::try_from(current_account_id).unwrap();
        self.token.mint(
            token_id.clone(),
            escrow_id,
            Some(TokenMetadata {
                title: None,
                description: None,
                media: Some(media.clone()),
                media_hash: Some(media_hash),
                copies: Some(1),
                issued_at: Some(env::block_timestamp().to_string()),
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None,
            }),
        );
        env::promise_create(
            env::current_account_id(),
            b"nft_transfer",
            json!({
                "token_id": token_id,
                "receiver_id": owner_id,
            })
            .to_string()
            .as_bytes(),
            ONE_YOCTO,
            SINGLE_CALL_GAS,
        );
        media
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, token);
near_contract_standards::impl_non_fungible_token_approval!(Contract, token);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, token);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod unit {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;

    use super::*;

    const MINT_STORAGE_COST: u128 = 25900000000000000000000;
    // 0.0259
    const TOKEN_ID: &str = "bafkreib5ry3rranl7tqov2uilfxbg4chuy6w2px3shlbchysqbubnu2vqu";
    const AVATAR: &str = "svg+xml;base64,PHN2ZyB2aWV3Qm94PSItMzIgLTMyIDY0IDY0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxjaXJjbGUgY3g9IjAiIGN5PSIwIiBmaWxsPSIjZWVlZWVlIiByPSIzMiIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMCIgY3k9Ii0yNCIgZmlsbD0iIzIxNTRlZCIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMCIgY3k9Ii0xMiIgZmlsbD0iI2JmODlmNSIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iLTEwIiBjeT0iLTE4IiBmaWxsPSIjYTUwZDFkIiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSItMjAiIGN5PSItMTIiIGZpbGw9IiM4N2VkMjEiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9Ii0xMCIgY3k9Ii02IiBmaWxsPSIjMGRhNTMzIiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSItMjAiIGN5PSIwIiBmaWxsPSIjYTUwZDFkIiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSItMjAiIGN5PSIxMiIgZmlsbD0iIzIxNTRlZCIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iLTEwIiBjeT0iNiIgZmlsbD0iI2JmODlmNSIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iLTEwIiBjeT0iMTgiIGZpbGw9IiM4OWY1ZjUiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9IjAiIGN5PSIyNCIgZmlsbD0iIzA2NDcwZiIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMCIgY3k9IjEyIiBmaWxsPSIjNDcyYTA2IiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSIxMCIgY3k9IjE4IiBmaWxsPSIjMjY0NzA2IiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSIyMCIgY3k9IjEyIiBmaWxsPSIjMjUyMWVkIiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSIxMCIgY3k9IjYiIGZpbGw9IiM0NzA2MWYiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9IjIwIiBjeT0iMCIgZmlsbD0iIzI2NDcwNiIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMjAiIGN5PSItMTIiIGZpbGw9IiMwNjQ3MGYiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9IjEwIiBjeT0iLTYiIGZpbGw9IiM0NzJhMDYiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9IjEwIiBjeT0iLTE4IiBmaWxsPSIjODlmNWY1IiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSIwIiBjeT0iMCIgZmlsbD0iIzA2NDcyZSIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PC9zdmc+";

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new();
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_avatar_create() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .signer_account_id(accounts(0))
            .build());

        let avatar = contract.avatar_create();
        assert_eq!(avatar, AVATAR);

        let token = contract.nft_token(TOKEN_ID.to_string()).unwrap();
        assert_eq!(token.owner_id, accounts(0).to_string());
        let avatar = token.metadata.unwrap().media.unwrap();
        assert_eq!(avatar, AVATAR);
        assert_eq!(token.approved_account_ids.unwrap().len(), 0);
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        contract.avatar_create();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), TOKEN_ID.to_string(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(TOKEN_ID.to_string()) {
            assert_eq!(token.token_id, TOKEN_ID.to_string());
            assert_eq!(token.owner_id, accounts(1).to_string());
            assert_eq!(token.metadata.unwrap().media.unwrap().len(), 1711);
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        contract.avatar_create();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(TOKEN_ID.to_string(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(TOKEN_ID.to_string(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        contract.avatar_create();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(TOKEN_ID.to_string(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(TOKEN_ID.to_string(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(TOKEN_ID.to_string(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        contract.avatar_create();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(TOKEN_ID.to_string(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(TOKEN_ID.to_string());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(TOKEN_ID.to_string(), accounts(1), Some(1)));
    }
}
