//! Ownable contract.
//! The logic was based off of: https://github.com/OpenZeppelin/rust-contracts-stylus/blob/main/contracts/src/access/ownable.rs

#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;
extern crate initialization;

use alloc::{vec, vec::Vec};
use alloy_primitives::U256;
use initialization::Initialization;
use stylus_sdk::{
    alloy_primitives::Address,
    alloy_sol_types::sol,
    call::Call,
    contract, evm,
    prelude::{entrypoint, external, sol_interface, sol_storage, SolidityError},
};

sol_interface! {
    interface ISeabrick {
        function mint(address to) external returns(uint256);
    }

    interface IERC20 {
        function transfer(address to, uint256 value) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}

sol! {
    /// Emitted when contract sell a NFT
    #[allow(missing_docs)]
    event NFTSold(address indexed owner, uint256 indexed id);
}

sol! {
    /// Not enought approved currencies
    #[derive(Debug)]
    #[allow(missing_docs)]
    error NotEnoughtCurrencies(uint256 required, uint256 current);

    #[derive(Debug)]
    #[allow(missing_docs)]
    error NotBought();
}

#[derive(SolidityError, Debug)]
pub enum MarketError {
    NeedAllowance(NotEnoughtCurrencies),
    NotBought(NotBought),
}

sol_storage! {
    #[entrypoint]
    pub struct Market {
        /// Total NFTs sold
        uint256 total_sold;
        /// Total tokens collected (total currency)
        uint256 total_collected;
        /// ERC20 Contract to use a paymnet
        address payment_token;
        /// Contract NFT address
        address nft_token;
        /// USD Price per NFT (should include the decimals)
        uint256 price;


        Initialization init;
    }
}

#[external]
impl Market {
    pub fn initialization(
        &mut self,
        token: Address,
        payment_token: Address,
        price: U256,
    ) -> Result<(), Vec<u8>> {
        // Check if already init. Revert if already init
        self.init._check_init()?;

        // Set token contract
        self.nft_token.set(token);

        // Set payment_token contract
        self.payment_token.set(payment_token);

        // Set NFT price
        self.price.set(price);

        // Change contract state to already initialized
        self.init.is_init.set(true);

        Ok(())
    }

    pub fn buy(&mut self, buyer: Address) -> Result<(), MarketError> {
        let payment_token = self._get_payment_token();
        let price = self.price.get();

        let tokens_allowed = payment_token
            .allowance(Call::new_in(self), buyer, contract::address())
            .unwrap();

        if tokens_allowed < price {
            return Err(MarketError::NeedAllowance(NotEnoughtCurrencies {
                required: price,
                current: U256::from(0),
            }));
        }

        let success = payment_token
            .transfer_from(Call::new_in(self), buyer, contract::address(), price)
            .unwrap();

        if success {
            let nft = self._get_nft_token();
            let new_id = nft.mint(Call::new_in(self), buyer).unwrap();

            evm::log(NFTSold {
                owner: buyer,
                id: new_id,
            });
        } else {
            return Err(MarketError::NotBought(NotBought {}));
        }

        Ok(())
    }
}

impl Market {
    pub fn _get_nft_token(&self) -> ISeabrick {
        return ISeabrick::new(self.nft_token.get());
    }

    pub fn _get_payment_token(&self) -> IERC20 {
        return IERC20::new(self.payment_token.get());
    }
}
