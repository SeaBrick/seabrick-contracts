#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;
mod initialization;
mod ownable;

use alloc::{vec, vec::Vec};
use initialization::Initialization;
use ownable::Ownable;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    alloy_sol_types::sol,
    call::Call,
    contract, evm, msg,
    prelude::{entrypoint, public, sol_interface, sol_storage, SolidityError},
};

sol_interface! {
    interface ISeabrick {
        function mint(address to) external returns (uint256);
        function mintBatch(address to, uint8 amount) external;
        function totalSupply() external returns (uint256);
    }

    interface IERC20 {
        function decimals() external view returns (uint8);
        function transfer(address to, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }

    interface Ownership {
        function owner() external view returns (address);
    }

}

sol_interface! {
    interface AggregatorV3Interface {
        function decimals() external view returns (uint8);
        function latestRoundData() external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound);
    }
}

sol! {
    /// A chainlink aggregator added
    event AggregatorAdded(bytes32 name, address aggregator, address token);

    /// Emitted when contract sell a NFT
    event Buy(address buyer, uint256 id, uint256 amountSpent, bytes32 aggregator);

    /// Tokens claimed
    event Claimed(address token, uint256 amount, bytes32 aggregator);

    event SaleDetails(address nftAddress, uint256 price);
}

sol! {
    /// NFT not bought
    error PaymentFailed();

    /// Mismatch on aggregators data provided
    error MismatchAggregators();

    /// Error when claiming
    error ClaimFailed();

    /// Error when setting price as zero
    error ZeroPrice();
}

#[derive(SolidityError)]
pub enum MarketError {
    PaymentFailed(PaymentFailed),
    MismatchAggregators(MismatchAggregators),
    ClaimFailed(ClaimFailed),
    ZeroPrice(ZeroPrice),
}

sol_storage! {
    pub struct AggregatorInfo {
        /// The chainlink oracle aggregator address for the given token
        address agregator_address;

        /// Token that will be used as payment and that correspond to the oracle Token/USD
        /// If token address is a non-zero address, it will try to be used to make transfers.
        /// If this token address is a zero address, it will assume that is native currency (like ETH in Arbitrum One)
        address token;
    }

    #[entrypoint]
    pub struct Market {
        /// USD Price per NFT
        uint256 price;

        /// Contract NFT address
        address nft_token;

        /// Mapping for hashed names. Example keccak("ETH/USD") to his price feed aggregator address.
        /// Of course you can add any oracle address, but this code is intended to work only for USD based oracles like
        /// ETH/USD, ARB/USD, etc.
        mapping(bytes32 => AggregatorInfo) price_feeds;

        /// Total tokens collected (total currency)
        mapping(address => uint256) total_collected;

        #[borrow]
        Initialization init;

        #[borrow]
        Ownable ownable;
    }
}

impl Market {
    pub fn get_amount_price(&mut self, amount: u8, name: FixedBytes<32>) -> Result<U256, Vec<u8>> {
        let payment_token = IERC20::new(self.price_feeds.get(name).token.get());
        let oracle = AggregatorV3Interface::new(self.price_feeds.get(name).agregator_address.get());

        // Get latest answer price
        let latest_answer =
            U256::from_limbs(oracle.latest_round_data(Call::new_in(self))?.1.into_limbs());

        let oracle_decimals = U256::from(oracle.decimals(Call::new_in(self))?);

        let payment_decimals = U256::from(payment_token.decimals(Call::new_in(self))?);

        // Scaled price
        let usd_price = self.price.get()
            * U256::from(10).pow(payment_decimals)
            * U256::from(10).pow(oracle_decimals);

        let amount_need = usd_price.div_ceil(latest_answer);

        Ok(amount_need * U256::from(amount))
    }

    pub fn set_aggregators_internal(
        &mut self,
        names: Vec<FixedBytes<32>>,
        agregators: Vec<Address>,
        tokens: Vec<Address>,
    ) -> Result<(), Vec<u8>> {
        // Set agregators info
        if names.len() != agregators.len() || names.len() != tokens.len() {
            return Err(MarketError::MismatchAggregators(MismatchAggregators {}).into());
        }

        for i in 0..names.len() {
            let mut map_aggregator = self.price_feeds.setter(names[i]);
            map_aggregator.agregator_address.set(agregators[i]);
            map_aggregator.token.set(tokens[i]);

            evm::log(AggregatorAdded {
                name: names[i],
                aggregator: agregators[i],
                token: tokens[i],
            });
        }

        Ok(())
    }
}

#[public]
#[inherit(Ownable)]
impl Market {
    pub fn initialization(
        &mut self,
        ownership: Address,
        price: U256,
        nft_token: Address,
        names: Vec<FixedBytes<32>>,
        agregators: Vec<Address>,
        tokens: Vec<Address>,
    ) -> Result<(), Vec<u8>> {
        // Check if already init. Revert if already init
        self.init._check_init()?;

        // Set contract owner/claimer using the initializer deployer
        self.ownable.set_ownership_contract(ownership);

        // Set NFT price
        self.price.set(price);

        // Set NFT token contract
        self.nft_token.set(nft_token);

        // Add the agregators
        self.set_aggregators_internal(names, agregators, tokens)?;

        evm::log(SaleDetails {
            price,
            nftAddress: nft_token,
        });

        // Change contract state to already initialized
        self.init._set_init(true);

        Ok(())
    }

    pub fn set_agregators(
        &mut self,
        names: Vec<FixedBytes<32>>,
        agregators: Vec<Address>,
        tokens: Vec<Address>,
    ) -> Result<(), Vec<u8>> {
        self.ownable.only_owner()?;

        self.set_aggregators_internal(names, agregators, tokens)?;

        Ok(())
    }

    pub fn set_price(&mut self, price: U256) -> Result<(), Vec<u8>> {
        self.ownable.only_owner()?;

        if price == U256::ZERO {
            return Err(MarketError::ZeroPrice(ZeroPrice {}).into());
        }

        // Set NFT price
        self.price.set(price);

        Ok(())
    }

    pub fn buy(&mut self, buyer: Address, name: FixedBytes<32>, amount: u8) -> Result<(), Vec<u8>> {
        let payment_token = IERC20::new(self.price_feeds.get(name).token.get());

        let amount_needed = self.get_amount_price(amount, name)?;

        let success = payment_token.transfer_from(
            Call::new_in(self),
            buyer,
            contract::address(),
            amount_needed,
        )?;
        if !success {
            return Err(MarketError::PaymentFailed(PaymentFailed {}).into());
        }

        // Increasing the total collected by this payment token
        let collected = self.total_collected.get(payment_token.address);
        self.total_collected
            .setter(payment_token.address)
            .set(collected + amount_needed);

        let seabrick = ISeabrick::new(self.nft_token.get());

        if amount == 1 {
            // Mint the token to the buyer address
            let id = seabrick.mint(Call::new_in(self), buyer)?;

            evm::log(Buy {
                buyer,
                id,
                amountSpent: amount_needed,
                aggregator: name,
            });
        } else {
            let id_init = (seabrick.total_supply(Call::new_in(self))?) + U256::from(1u8);

            seabrick.mint_batch(Call::new_in(self), buyer, amount)?;

            for i in 0..amount.into() {
                evm::log(Buy {
                    buyer,
                    id: id_init + U256::from(i),
                    amountSpent: amount_needed,
                    aggregator: name,
                });
            }
        }

        Ok(())
    }

    pub fn claim(&mut self, name: FixedBytes<32>) -> Result<(), Vec<u8>> {
        self.ownable.only_owner()?;

        let claim_token = IERC20::new(self.price_feeds.get(name).token.get());
        let amount_collected = self.total_collected.get(claim_token.address);

        // Decreasing the total collected by this payment token to ZERO since everything was claimed
        self.total_collected
            .setter(claim_token.address)
            .set(U256::ZERO);

        let success = claim_token.transfer(Call::new_in(self), msg::sender(), amount_collected)?;
        if !success {
            return Err(MarketError::ClaimFailed(ClaimFailed {}).into());
        }

        evm::log(Claimed {
            token: claim_token.address,
            amount: amount_collected,
            aggregator: name,
        });

        Ok(())
    }
}
