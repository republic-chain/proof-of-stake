use super::{Address, Amount, Nonce};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    pub balance: Amount,
    pub nonce: Nonce,
    pub code: Vec<u8>, // For smart contracts
    pub storage: HashMap<[u8; 32], [u8; 32]>, // Contract storage
}

impl Account {
    pub fn new(address: Address, balance: Amount) -> Self {
        Account {
            address,
            balance,
            nonce: 0,
            code: Vec::new(),
            storage: HashMap::new(),
        }
    }

    pub fn is_contract(&self) -> bool {
        !self.code.is_empty()
    }

    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    pub fn debit(&mut self, amount: Amount) -> Result<(), String> {
        if self.balance < amount {
            return Err("Insufficient balance".to_string());
        }
        self.balance -= amount;
        Ok(())
    }

    pub fn credit(&mut self, amount: Amount) {
        self.balance += amount;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeInfo {
    pub amount: Amount,
    pub validator: Address,
    pub delegator: Address,
    pub rewards: Amount,
    pub unbonding_height: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountState {
    pub accounts: HashMap<Address, Account>,
    pub stakes: HashMap<Address, Vec<StakeInfo>>,
    pub total_supply: Amount,
}

impl AccountState {
    pub fn new() -> Self {
        AccountState {
            accounts: HashMap::new(),
            stakes: HashMap::new(),
            total_supply: 0,
        }
    }

    pub fn get_account(&self, address: &Address) -> Option<&Account> {
        self.accounts.get(address)
    }

    pub fn get_account_mut(&mut self, address: &Address) -> Option<&mut Account> {
        self.accounts.get_mut(address)
    }

    pub fn create_account(&mut self, address: Address, initial_balance: Amount) {
        let account = Account::new(address, initial_balance);
        self.accounts.insert(address, account);
        self.total_supply += initial_balance;
    }

    pub fn transfer(&mut self, from: &Address, to: &Address, amount: Amount) -> Result<(), String> {
        // Check if sender has sufficient balance
        {
            let sender = self.accounts.get(from).ok_or("Sender account not found")?;
            if sender.balance < amount {
                return Err("Insufficient balance".to_string());
            }
        }

        // Create recipient account if it doesn't exist
        if !self.accounts.contains_key(to) {
            self.create_account(*to, 0);
        }

        // Perform transfer
        let sender = self.accounts.get_mut(from).unwrap();
        sender.debit(amount)?;

        let recipient = self.accounts.get_mut(to).unwrap();
        recipient.credit(amount);

        Ok(())
    }

    pub fn stake(&mut self, delegator: Address, validator: Address, amount: Amount) -> Result<(), String> {
        // Check if delegator has sufficient balance
        {
            let account = self.accounts.get(&delegator).ok_or("Delegator account not found")?;
            if account.balance < amount {
                return Err("Insufficient balance".to_string());
            }
        }

        // Debit from delegator's account
        let account = self.accounts.get_mut(&delegator).unwrap();
        account.debit(amount)?;

        // Add stake info
        let stake_info = StakeInfo {
            amount,
            validator,
            delegator,
            rewards: 0,
            unbonding_height: None,
        };

        self.stakes.entry(delegator).or_insert_with(Vec::new).push(stake_info);

        Ok(())
    }

    pub fn unstake(&mut self, delegator: Address, validator: Address, amount: Amount, unbonding_height: u64) -> Result<(), String> {
        let stakes = self.stakes.get_mut(&delegator).ok_or("No stakes found for delegator")?;

        let mut remaining_amount = amount;
        for stake in stakes.iter_mut() {
            if stake.validator == validator && stake.unbonding_height.is_none() && remaining_amount > 0 {
                let unstake_amount = remaining_amount.min(stake.amount);
                stake.amount -= unstake_amount;
                stake.unbonding_height = Some(unbonding_height);
                remaining_amount -= unstake_amount;

                if stake.amount == 0 {
                    // Remove empty stake
                    continue;
                }
            }
        }

        if remaining_amount > 0 {
            return Err("Insufficient staked amount".to_string());
        }

        // Remove empty stakes
        stakes.retain(|stake| stake.amount > 0);

        Ok(())
    }

    pub fn get_total_staked(&self, delegator: &Address) -> Amount {
        self.stakes
            .get(delegator)
            .map(|stakes| {
                stakes
                    .iter()
                    .filter(|stake| stake.unbonding_height.is_none())
                    .map(|stake| stake.amount)
                    .sum()
            })
            .unwrap_or(0)
    }

    pub fn get_validator_total_stake(&self, validator: &Address) -> Amount {
        self.stakes
            .values()
            .flatten()
            .filter(|stake| stake.validator == *validator && stake.unbonding_height.is_none())
            .map(|stake| stake.amount)
            .sum()
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self::new()
    }
}