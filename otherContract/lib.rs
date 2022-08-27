#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;
//use erc20::Erc20Ref;

// contract ERC20Interface {
//     // Storage Getters
//     function totalSupply() public view returns (uint);
//     function balanceOf(address tokenOwner) public view returns (uint balance);
//     function allowance(address tokenOwner, address spender) public view returns (uint remaining);

//     // Public Functions
//     function transfer(address to, uint tokens) public returns (bool success);
//     function approve(address spender, uint tokens) public returns (bool success);
//     function transferFrom(address from, address to, uint tokens) public returns (bool success);

//     // Contract Events
//     event Transfer(address indexed from, address indexed to, uint tokens);
//     event Approval(address indexed tokenOwner, address indexed spender, uint tokens);
// }

#[ink::contract]
mod otherContract {

    use erc20::Erc20Ref;

    use ink_storage::{
        traits::SpreadAllocate,
        //Mapping,
    };

    #[cfg(not(feature = "ink-as-dependency"))]
    

    /// Create storage for a simple ERC-20 contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct OtherContract {
        address_from: AccountId,
        address_to: AccountId,

        erc20: Erc20Ref
    }

    /// Specify ERC-20 error type.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Return if the balance cannot fulfill a request.
        InsufficientBalance,
        InsufficientAllowance,
        InvalidToAddress,
        InvalidFromAddress
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

    impl OtherContract {
        /// Create a new ERC-20 contract with an initial supply.
        #[ink(constructor)]
        pub fn new(erc20_code_hash: Hash, address_from: AccountId, address_to: AccountId) -> Self {

            // Initialize mapping for the contract.
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, erc20_code_hash, address_from, address_to)
            })
        }

        /// Initialize the ERC-20 contract with the specified initial supply.
        fn new_init(&mut self, erc20_code_hash: Hash, address_from: AccountId, address_to: AccountId) {
            let caller = Self::env().caller();
            let total_balance = Self::env().balance();
            let erc20 = Erc20Ref::new(1337)
            .endowment(total_balance / 4)
            .code_hash(erc20_code_hash)
            .instantiate()
            .expect("failed at instantiating the `OtherContract` contract");
            self.erc20 = &erc20;
            self.address_from = address_from;
            self.address_to = address_to;
            // Self {
            //     address_from,
            //     address_to,
            //     erc20
            // }
        }

        /// Calls the erc20 contract.

        #[ink(message)]
        pub fn deposit(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            //TODO: change address from to array
            if caller != self.address_from {
                return Err(Error::InvalidFromAddress)
            }
            //Check caller balance
            let caller_balance = self.erc20.balanceOf(&caller);
            if caller_balance < value {
                return Err(Error::InsufficientBalance)
            }

            //Check caller approve allowance balance
            let contract_address = self.env().account_id();
            let caller_approved = self.erc20.allowance(&caller, &contract_address);
            if caller_approved < value {
                return Err(Error::InsufficientAllowance)
            }

            //Transfer token to contract
            self.erc20.transferFrom(&caller, &contract_address, value)
        }     
    }
        //Cross contract test unavailable now

        // #[cfg(test)]
        // mod tests {
        // use super::*;
    
        // use ink_lang as ink;
    
        // #[ink::test]
        // fn new_works() {
        //     let erc20 = 
        //     let contract = OtherContract::new(777);
        //     assert_eq!(contract.total_supply(), 777);
        // }
}