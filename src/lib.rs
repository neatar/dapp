mod identicon;

use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{hash_account_id, NonFungibleToken};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use std::convert::TryFrom;
use std::convert::TryInto;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token_last_id: u128,
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

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
        let owner_id = ValidAccountId::try_from(env::signer_account_id()).unwrap();
        Self {
            token_last_id: 0,
            tokens: NonFungibleToken::new(
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
                    icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                    base_uri: None,
                    reference: None,
                    reference_hash: None,
                }),
            ),
        }
    }

    pub fn avatar_of(self, account_id: ValidAccountId) -> String {
        let list = self.tokens.nft_tokens_for_owner(account_id, None, None);
        if list.is_empty() {
            return String::new();
        }
        list.first()
            .cloned()
            .unwrap()
            .metadata
            .unwrap()
            .media
            .unwrap()
    }

    #[payable]
    pub fn avatar_create(&mut self) -> Token {
        let owner_id = env::signer_account_id().try_into().unwrap();
        self.avatar_create_for(owner_id)
    }

    #[payable]
    pub fn avatar_create_for(&mut self, owner_id: ValidAccountId) -> Token {
        let data_image = format!(
            "data:image/svg+xml;base64,{}",
            identicon::make(&hash_account_id(&owner_id.clone().into()))
        );
        self.tokens.owner_id = env::signer_account_id(); // FIXME
        let token = self.tokens.mint(
            self.token_last_id.to_string(),
            owner_id,
            Some(TokenMetadata {
                title: None,
                description: None,
                media: Some(data_image),
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None,
            }),
        );
        self.token_last_id += 1;
        token
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;

    const MINT_STORAGE_COST: u128 = 22650000000000000000000; // 0.02265

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

        let token_id = "0".to_string();
        let token = contract.avatar_create();
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id, accounts(0).to_string());
        assert_eq!(token.metadata.unwrap().media.unwrap().len(), 1750);
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
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
        let token_id = "0".to_string();
        contract.avatar_create();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id, accounts(1).to_string());
            assert_eq!(token.metadata.unwrap().media.unwrap().len(), 1750);
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
        let token_id = "0".to_string();
        contract.avatar_create();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id, accounts(1), Some(1)));
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
        let token_id = "0".to_string();
        contract.avatar_create();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id, accounts(1), None));
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
        let token_id = "0".to_string();
        contract.avatar_create();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id, accounts(1), Some(1)));
    }
}
