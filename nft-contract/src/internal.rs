use crate::*;

#[near_bindgen]
impl Contract {
    pub(crate) fn internal_add_token_to_owner(
        &mut self,
        token_id: &TokenId,
        account_id: &AccountId,
    ) {
        let mut tokens_set = self.tokens_per_owner.get(&account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::TokenPerOwnerInnerKey {
                    account_id_hash: hash_account_id(account_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });
        tokens_set.insert(token_id);
        self.tokens_per_owner.insert(account_id, &tokens_set);
    }

    pub(crate) fn internal_remove_token_from_owner(
        &mut self,
        token_id: &TokenId,
        account_id: &AccountId,
    ) {
        let mut tokens_set = self
            .tokens_per_owner
            .get(&account_id)
            .expect("Token should be owned by owner");
        tokens_set.remove(token_id);
        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(account_id);
        } else {
            self.tokens_per_owner.insert(account_id, &tokens_set);
        }
    }

    // return laij data token cũ trước khi thực hiện transfer
    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Token {
        let old_token = self.tokens_by_id.get(token_id).expect("Not found token");
        let token = self.tokens_by_id.get(token_id).expect("Not found token");
        if sender_id != &token.owner_id {
            if !token.approved_account_ids.contains_key(sender_id) {
                env::panic(b"Unauthorized");
            }
            if let Some(enforced_approved_id) = approval_id {
                let actual_approval_id = token
                    .approved_account_ids
                    .get(sender_id)
                    .expect("Sender is not approved account");

                assert_eq!(
                    actual_approval_id, &enforced_approved_id,
                    "The actual approval id {} is different from the given approval id {} ",
                    actual_approval_id, enforced_approved_id
                );
            }
        };

        assert_ne!(
            &token.owner_id, receiver_id,
            "The token owner and the receiver should be different"
        );

        self.internal_remove_token_from_owner(&token_id, &token.owner_id);
        self.internal_add_token_to_owner(&token_id, receiver_id);
        let new_token = Token {
            owner_id: receiver_id.clone(),
            approved_account_ids: HashMap::new(),
            next_approval_id: token.next_approval_id,
            royalty: token.royalty.clone(),
            users: token.users,
        };
        self.tokens_by_id.insert(token_id, &new_token);

        if let Some(memo) = &memo {
            log!("Memo {}", memo);
        }

        let mut authorized_id = None;
        if approval_id.is_some() {
            authorized_id = Some(sender_id.to_string());
        }

        let nft_transfer_log: EventLog = EventLog {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                authorized_id,
                old_owner_id: token.owner_id.clone().to_string(),
                new_owner_id: receiver_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo,
            }]),
        };

        env::log(&nft_transfer_log.to_string().as_bytes());

        // TODO: refactor to avoid reinitiate old token to return

        old_token
    }

    pub(crate) fn internal_add_users(
        &mut self,
        admin_id: AccountId,
        user_id: AccountId,
        token_id: &TokenId,
    ) -> Token {
        let mut token = self.tokens_by_id.get(token_id).expect("Not found token");
        if admin_id.to_string() != token.owner_id {
            if !token.approved_account_ids.contains_key(&admin_id) {
                env::panic(b"Unauthorized");
            }
        };
        assert_ne!(
            token.owner_id, user_id,
            "The token owner and the receiver should be different"
        );

        let storage_used = bytes_for_approved_account_id(&user_id);
        token.users.insert(&user_id);
        self.tokens_by_id.insert(token_id, &token);
        // refund_deposit(storage_used);
        token
    }
}
