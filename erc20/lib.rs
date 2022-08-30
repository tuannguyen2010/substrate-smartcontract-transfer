#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };

    /// Create storage for a simple ERC-20 contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Erc20 {
        /// Total token supply.
        total_supply: Balance,
        /// Mapping from owner to number of owned tokens.
        balances: Mapping<AccountId, Balance>,

        /// Balances that can be transferred by non-owners: (owner, spender) -> allowed
        allowances: Mapping<(AccountId, AccountId), Balance>,

        allowances_with: Mapping<(AccountId, AccountId), Timestamp>,
    }

    /// Specify ERC-20 error type.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Return if the balance cannot fulfill a request.
        InsufficientBalance,
        InsufficientAllowance,
        InvalidTime
    }

    /// Specify the ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct Transfer {
       #[ink(topic)]
       from: Option<AccountId>,
       #[ink(topic)]
       to: Option<AccountId>,
       value: Balance,
     }

     #[ink(event)]
     pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
     }

    impl Erc20 {
        /// Create a new ERC-20 contract with an initial supply.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance, to: AccountId, condition: Timestamp) -> Self {
            // Initialize mapping for the contract.
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, initial_supply, to, condition)
            })
        }

        /// Initialize the ERC-20 contract with the specified initial supply.
        fn new_init(&mut self, initial_supply: Balance, to: AccountId, condition: Timestamp) {
            let caller = Self::env().caller();
            self.balances.insert(&caller, &initial_supply);
            self.total_supply = initial_supply;
            
            //TODO HERE
            //Allow user B get token from user A
            self.allowances.insert((&caller, &to), &initial_supply);
            self.allowances_with.insert((&caller, &to), &condition);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
              });
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
         ) -> Result<()> {
             let from_balance = self.balance_of_impl(from);
             if from_balance < value {
                 return Err(Error::InsufficientBalance)
             }
         
             self.balances.insert(from, &(from_balance - value));
             let to_balance = self.balance_of_impl(to);
             self.balances.insert(to, &(to_balance + value));
             self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
             });
             Ok(())
         }

         #[inline]
         fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
         }

         #[ink(message)]
         pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
              owner,
              spender,
              value,
            });
            Ok(())
         }

         #[ink(message)]
         pub fn approve_with(&mut self, spender: AccountId, value: Balance, open_time: Timestamp) -> Result<()> {
            let owner = self.env().caller();
           
            self.allowances.insert((&owner, &spender), &value);
            self.allowances_with.insert((&owner, &spender), &open_time);

            self.env().emit_event(Approval {
              owner,
              spender,
              value,
            });
            Ok(())
         }

         #[ink(message)]
         pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
         }

         //TODO: get condition value
         #[ink(message)]
         pub fn allowances_with(&self, owner: AccountId, spender: AccountId) -> Timestamp {
            self.allowances_with_impl(&owner, &spender)
         }

         /// Transfers tokens on the behalf of the `from` account to the `to account 
         #[ink(message)]
         pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
         ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(&from, &to, value)?;
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
           }


         
           #[ink(message)]
           pub fn transfer_from_with(
              &mut self,
              from: AccountId,
              to: AccountId,
              value: Balance,
           ) -> Result<()> {
              let caller = self.env().caller();
              let allowance = self.allowance_impl(&from, &caller);
              if allowance < value {
                  return Err(Error::InsufficientAllowance)
              }

              //TODO: add transfer condition
              let condition = self.allowances_with_impl(&from, &caller);
              if condition >  Self::env().block_timestamp() {
                return Err(Error::InvalidTime)
              }

              self.transfer_from_to(&from, &to, value)?;
              self.allowances
                  .insert((&from, &caller), &(allowance - value));
            
                  self.allowances_with
                  .insert((&from, &caller), &(Self::env().block_timestamp()));
              Ok(())
             }

         #[inline]
         fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
         }

         //TODO: add get condition value
         #[inline]
         fn allowances_with_impl(&self, owner: &AccountId, spender: &AccountId) -> Timestamp {
            self.allowances_with.get((owner, spender)).unwrap_or_default()
         }

    }

        #[cfg(test)]
        mod tests {
        use super::*;
    
        use ink_lang as ink;
    
        #[ink::test]
        fn new_works() {
            let contract = Erc20::new(777);
            assert_eq!(contract.total_supply(), 777);
        }
    
        #[ink::test]
        fn balance_works() {
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
           let mut erc20 = Erc20::new(100);
           assert_eq!(erc20.balance_of(AccountId::from([0x0; 32])), 0);
           assert_eq!(erc20.transfer((AccountId::from([0x0; 32])), 10), Ok(()));
           assert_eq!(erc20.balance_of(AccountId::from([0x0; 32])), 10);
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 20);
            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
         }

        #[ink::test]
        fn allowances_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 200);
            assert_eq!(contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])), 200);
         
         contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 50);
         assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
         assert_eq!(contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])), 150);
         
         contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 100);
         assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
         assert_eq!(contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])), 150);
         }
    }
}