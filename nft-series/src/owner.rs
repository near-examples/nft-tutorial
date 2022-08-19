use crate::*;

#[near_bindgen]
impl Contract {
    /// approved minters
    pub fn add_approved_minter(&mut self, account_id: AccountId) {
        self.assert_contract_owner();
        self.approved_minters.insert(&account_id);
    }

    pub fn remove_approved_minter(&mut self, account_id: AccountId) {
        self.assert_contract_owner();
        self.approved_minters.remove(&account_id);
    }

    pub fn is_approved_minter(&self, account_id: AccountId) -> bool {
        self.approved_minters.contains(&account_id)
    }

    /// approved creators
    pub fn add_approved_creator(&mut self, account_id: AccountId) {
        self.assert_contract_owner();
        self.approved_creators.insert(&account_id);
    }

    pub fn remove_approved_creator(&mut self, account_id: AccountId) {
        self.assert_contract_owner();
        self.approved_creators.remove(&account_id);
    }

    pub fn is_approved_creator(&self, account_id: AccountId) -> bool {
        self.approved_creators.contains(&account_id)
    }
}
