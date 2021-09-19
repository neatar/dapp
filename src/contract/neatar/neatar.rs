use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use near_contract_standards::non_fungible_token::hash_account_id;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::Base64VecU8;
use near_sdk::log;
use near_sdk::require;
use near_sdk::serde_json::json;
use near_sdk::Balance;
use near_sdk::Gas;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

mod identicon;

fn pack_data_image(data: String, media_type: Option<String>) -> String {
    format!(
        "{};base64,{}",
        media_type.unwrap_or_else(|| "svg+xml".to_string()),
        base64::encode(data)
    )
}

fn new_token(svg: String, owner_id: Option<AccountId>) -> Token {
    let hash = Code::Sha2_256.digest(svg.as_bytes());
    let token_id = Cid::new_v1(RAW, hash).to_string();
    let owner_id = owner_id.unwrap_or_else(env::current_account_id);
    Token {
        token_id,
        owner_id,
        metadata: Some(new_token_metadata(svg)),
        approved_account_ids: None,
    }
}

fn new_token_metadata(svg: String) -> TokenMetadata {
    let media = pack_data_image(svg.clone(), None);
    let media_hash = Base64VecU8(env::sha256(svg.as_bytes()));
    TokenMetadata {
        title: None,
        description: None,
        media: Some(media),
        media_hash: Some(media_hash),
        copies: Some(1),
        issued_at: Some(env::block_timestamp().to_string()),
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    }
}

// Prepaid gas for making a single simple call.
const SINGLE_CALL_GAS: Gas = Gas(200000000000000);
const ONE_YOCTO: Balance = 1;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Neatar {
    token: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

const RAW: u64 = 0x55;
lazy_static_include::lazy_static_include_str! {
    LOGO => "../../web/asset/logo.svg",
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Neatar {
    #[private]
    #[init]
    pub fn init() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self::new()
    }

    fn new() -> Self {
        let owner_id = env::current_account_id();
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
                    name: "The Web3 avatar".to_string(),
                    symbol: "NEATAR".to_string(),
                    icon: Some(format!(
                        "data:image/{}",
                        pack_data_image(LOGO.to_string(), None)
                    )),
                    base_uri: Some("data:image".to_string()),
                    reference: None,
                    reference_hash: None,
                }),
            ),
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let current: Neatar = env::state_read().expect("State doesn't exist");
        let mut next = Neatar::new();
        next.token = current.token;
        next.metadata = current.metadata;
        next
    }

    fn current_token(&self, account_id: AccountId) -> Token {
        let list = self.token.nft_tokens_for_owner(account_id, None, None);
        if list.is_empty() {
            return new_token(LOGO.to_string(), None);
        }
        list.last().cloned().unwrap()
    }

    pub fn ft_burn(&mut self, token_id: TokenId) {
        let initial_storage_usage = env::storage_usage();
        let owner_id = self
            .token
            .owner_by_id
            .get(&token_id)
            .expect("Not found token");
        require!(owner_id == env::predecessor_account_id(), "Only owner");
        // make burn token
        match self
            .token
            .tokens_per_owner
            .as_mut()
            .and_then(|per_owner| per_owner.remove(&owner_id))
        {
            None => {}
            Some(mut set) => set.clear(),
        };
        self.token
            .approvals_by_id
            .as_mut()
            .and_then(|by_id| by_id.remove(&token_id));
        self.token
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.remove(&token_id));
        self.token.owner_by_id.remove(&token_id);
        // make refund for storage free
        let storage_free = initial_storage_usage
            .checked_sub(env::storage_usage())
            .unwrap_or_default();
        log!("storage free: {}", storage_free);
        let refund = env::storage_byte_cost() * Balance::from(storage_free);
        if refund > 1 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    }

    pub fn avatar_of(&self, account_id: AccountId) -> String {
        let token = self.current_token(account_id);
        let media = token.metadata.unwrap().media.unwrap();
        format!("data:image/{}", media)
    }

    #[payable]
    pub fn avatar_create(&mut self) -> String {
        let owner_id = env::signer_account_id();
        self.avatar_create_for(owner_id)
    }

    pub fn avatar_burn(&mut self) {
        self.ft_burn(self.current_token(env::predecessor_account_id()).token_id)
    }

    #[private]
    pub fn avatar_burn_for(&mut self, owner_id: AccountId) {
        // TODO
    }

    #[payable]
    #[private]
    pub fn avatar_create_for(&mut self, owner_id: AccountId) -> String {
        let initial_storage_usage = env::storage_usage();
        let hash: &[u8] = &hash_account_id(&owner_id);
        let svg = identicon::make(hash);
        let contract_id = env::current_account_id();
        let token = new_token(svg, None);
        let token_id = token.token_id;
        let metadata = token.metadata.unwrap();
        let media = metadata.media.clone().unwrap_or_default();
        self.token.owner_id = env::predecessor_account_id(); // FIXME
        self.token
            .mint(token_id.clone(), contract_id.clone(), Some(metadata));
        env::promise_create(
            contract_id,
            "nft_transfer",
            json!({
                "token_id": token_id,
                "receiver_id": owner_id,
            })
            .to_string()
            .as_bytes(),
            ONE_YOCTO,
            SINGLE_CALL_GAS,
        );
        let storage_usage = env::storage_usage()
            .checked_sub(initial_storage_usage)
            .unwrap_or_default();
        log!("storage usage: {}", storage_usage);
        media
    }
}

near_contract_standards::impl_non_fungible_token_core!(Neatar, token);
near_contract_standards::impl_non_fungible_token_approval!(Neatar, token);
near_contract_standards::impl_non_fungible_token_enumeration!(Neatar, token);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Neatar {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod unit {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    const MINT_STORAGE_COST: u128 = 25900000000000000000000;
    // 0.0259
    const TOKEN_ID: &str = "bafkreib5ry3rranl7tqov2uilfxbg4chuy6w2px3shlbchysqbubnu2vqu";
    const AVATAR: &str = "svg+xml;base64,PHN2ZyB2aWV3Qm94PSItMzIgLTMyIDY0IDY0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxjaXJjbGUgY3g9IjAiIGN5PSIwIiBmaWxsPSIjZWVlZWVlIiByPSIzMiIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMCIgY3k9Ii0yNCIgZmlsbD0iIzIxNTRlZCIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMCIgY3k9Ii0xMiIgZmlsbD0iI2JmODlmNSIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iLTEwIiBjeT0iLTE4IiBmaWxsPSIjYTUwZDFkIiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSItMjAiIGN5PSItMTIiIGZpbGw9IiM4N2VkMjEiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9Ii0xMCIgY3k9Ii02IiBmaWxsPSIjMGRhNTMzIiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSItMjAiIGN5PSIwIiBmaWxsPSIjYTUwZDFkIiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSItMjAiIGN5PSIxMiIgZmlsbD0iIzIxNTRlZCIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iLTEwIiBjeT0iNiIgZmlsbD0iI2JmODlmNSIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iLTEwIiBjeT0iMTgiIGZpbGw9IiM4OWY1ZjUiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9IjAiIGN5PSIyNCIgZmlsbD0iIzA2NDcwZiIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMCIgY3k9IjEyIiBmaWxsPSIjNDcyYTA2IiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSIxMCIgY3k9IjE4IiBmaWxsPSIjMjY0NzA2IiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSIyMCIgY3k9IjEyIiBmaWxsPSIjMjUyMWVkIiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSIxMCIgY3k9IjYiIGZpbGw9IiM0NzA2MWYiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9IjIwIiBjeT0iMCIgZmlsbD0iIzI2NDcwNiIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PGNpcmNsZSBjeD0iMjAiIGN5PSItMTIiIGZpbGw9IiMwNjQ3MGYiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9IjEwIiBjeT0iLTYiIGZpbGw9IiM0NzJhMDYiIHI9IjUiIHN0cm9rZT0ibm9uZSIvPjxjaXJjbGUgY3g9IjEwIiBjeT0iLTE4IiBmaWxsPSIjODlmNWY1IiByPSI1IiBzdHJva2U9Im5vbmUiLz48Y2lyY2xlIGN4PSIwIiBjeT0iMCIgZmlsbD0iIzA2NDcyZSIgcj0iNSIgc3Ryb2tlPSJub25lIi8+PC9zdmc+";

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
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
        let contract = Neatar::new();
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Neatar::default();
    }

    #[test]
    fn test_avatar_create() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Neatar::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .signer_account_id(accounts(0))
            .build());

        let avatar = contract.avatar_create();
        assert_eq!(avatar, AVATAR);

        let token = contract.nft_token(TOKEN_ID.to_string()).unwrap();
        assert_eq!(token.owner_id, accounts(0));
        let avatar = token.metadata.unwrap().media.unwrap();
        assert_eq!(avatar, AVATAR);
        assert_eq!(token.approved_account_ids.unwrap().len(), 0);
    }

    #[test]
    fn test_avatar_burn() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Neatar::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .signer_account_id(accounts(0))
            .build());

        assert_eq!(AVATAR, contract.avatar_create());
        assert_eq!(1722, contract.avatar_of(accounts(0)).len());
        contract.avatar_burn();
        assert_eq!(614, contract.avatar_of(accounts(0)).len());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Neatar::new();

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
            assert_eq!(token.owner_id, accounts(1));
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
        let mut contract = Neatar::new();

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
        let mut contract = Neatar::new();

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
        let mut contract = Neatar::new();

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
