#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod synthetics {
    use ink_storage::traits::SpreadLayout;

    /// ERC20 token like to represent long and shorts, TODO maybe use main ERC20
    #[derive(Debug, SpreadLayout)]
    #[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout)
    )]
    struct Token {
        total_supply: Balance,
        /// The balance of each user.
        balances: ink_storage::collections::HashMap<AccountId, Balance>
    }

    // TODO: refactor by using ERC20 trait?
    impl Token {
        pub fn new(initial_supply: Balance, owner: AccountId) -> Self {
            let mut balances = ink_storage::collections::HashMap::new();
            balances.insert(owner, initial_supply);

            Self {
                total_supply: initial_supply,
                balances
            }
        }

        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }
    }
    
    /// The actual contract
    #[ink(storage)]
    pub struct Synthetics {
        /// The short token, you first mint this to create the short
        short: Token,
        /// The long token, you ming this when you buy the corresponding short
        /// long total balance has to be less than or equal than the total balance of short
        long: Token,
        // Collateral balance, TODO
    }

    #[ink(event)]
    pub struct TokenMinted {
        #[ink(topic)]
        creator: AccountId,
        #[ink(topic)]
        value: Balance
    }

    impl Synthetics {
        #[ink(constructor)]
        /// When a contract is first created, a short is minted with the balance given
        pub fn new(initial_supply: Balance) -> Self {
            // TODO: how do I add the margin, start by taking the margin here
            let caller = Self::env().caller();
            let short = Token::new(initial_supply, caller);
            Self::env().emit_event(TokenMinted {
                creator: caller,
                value: initial_supply,
            });
            let long = Token::new(0, caller);

            Self{ short, long }
        }

        #[ink(message)]
        pub fn short_total_balance(&self) -> Balance {
            self.short.total_supply()
        }

        #[ink(message)]
        pub fn long_total_balance(&self) -> Balance {
            self.long.total_supply()
        }

        #[ink(message)]
        pub fn short_balance_of(&self, owner: AccountId) -> Balance {
            self.short.balance_of(owner)
        }

        #[ink(message)]
        pub fn long_balance_of(&self, owner: AccountId) -> Balance {
            self.long.balance_of(owner)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        /// We test if the constructor does its job.
        #[ink::test]
        fn constructur_works() {
            let synthetics = Synthetics::new(100);

            // TODO test events
            assert_eq!(synthetics.short_total_balance(), 100);
            assert_eq!(synthetics.long_total_balance(), 0);

            let account = AccountId::from([0x01; 32]);

            assert_eq!(synthetics.short_balance_of(account), 100);
            assert_eq!(synthetics.long_balance_of(account), 0);
        }
    }
}
