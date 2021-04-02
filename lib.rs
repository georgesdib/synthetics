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
        long: Token
    }

    #[ink(event)]
    /// Emitted when a Token is minted
    pub struct TokenMinted {
        #[ink(topic)]
        creator: AccountId,
        #[ink(topic)]
        value: Balance
    }

    // TODO: make this call an oracle, check return type, add asset type
    pub fn get_price() -> u128 {
        1
    }

    impl Synthetics {
        #[ink(constructor, selector = "0xCAFEBABE")]
        /// When a contract is first created, a short is minted with the balance given
        pub fn new(initial_supply: Balance) -> Self {
            // TODO: how do I add the margin, start by taking the margin here
            let caller = Self::env().caller();
            // Take the IM, if this fails don't continue the rest
            // TODO make the HC percentage a parameter
            let im = initial_supply * get_price() / 5;
            // TODO Should I panic?
            assert!(im <= Self::env().transferred_balance());
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

        use ink_env::{
            call,
            test,
        };
        use ink_lang as ink;

        /// We test if the constructor does its job.
        #[ink::test]
        fn constructur_works() {
            let account = create_contract(100);
            let synthetics = Synthetics::new(100);

            // TODO test events
            assert_eq!(synthetics.short_total_balance(), 100);
            assert_eq!(synthetics.long_total_balance(), 0);

            assert_eq!(synthetics.short_balance_of(account), 100);
            assert_eq!(synthetics.long_balance_of(account), 0);
        }

        /// Failure for not enough balance
        #[ink::test]
        #[should_panic]
        fn constructur_fails_no_balance() {
            create_contract(1);
            Synthetics::new(100);
        }


        // Helper functions
        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Off-chain environment should have been initialized already")
        }

        fn contract_id() -> AccountId {
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>(
            )
            .expect("Cannot get contract id")
        }

        fn set_sender(sender: AccountId) {
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            test::push_execution_context::<Environment>(
                sender,
                callee,
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }

        fn create_contract(initial_balance: Balance) -> AccountId {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.alice,
                contract_id(),
                1000000,
                initial_balance,
                data,
            );
            accounts.alice
        }
    }
}
