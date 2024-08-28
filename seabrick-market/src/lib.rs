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
    prelude::{entrypoint, external, sol_interface, sol_storage, SolidityError},
};

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

sol_interface! {
    interface ISeabrick {
        function totalSupply() external view returns (uint256);
        function mint(address to) external;
    }

    interface IERC20 {
        function decimals() external view returns (uint8);
        function transfer(address to, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }

}

sol_interface! {
    interface AggregatorV3Interface {
        function decimals() external view returns (uint8);
        function latestRoundData() external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound);
    }
}

sol! {
    event AggregatorAdded(bytes32 indexed name, address indexed aggregator, address indexed token);

    /// Emitted when contract sell a NFT
    event NFTSold(address indexed owner, uint256 indexed id);

    event TokenClaimed(address indexed token, uint256 indexed amount);
}

sol! {
    /// NFT not bought
    error NotBought();
    /// Mismatch on data provided
    error MismatchData();

    /// Error from a call
    error CallError(uint256 id);

    /// Error when claiming
    error NotClaimed();
}

#[derive(SolidityError)]
pub enum MarketError {
    NotBought(NotBought),
    MismatchData(MismatchData),
    CallError(CallError),
    NotClaimed(NotClaimed),
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

#[external]
#[inherit(Ownable)]
impl Market {
    pub fn initialization(
        &mut self,
        price: U256,
        nft_token: Address,
        names: Vec<FixedBytes<32>>,
        agregators: Vec<Address>,
        tokens: Vec<Address>,
    ) -> Result<(), MarketError> {
        // Check if already init. Revert if already init
        if let Err(_) = self.init._check_init() {
            return Err(MarketError::CallError(CallError { id: U256::from(1) }));
        }

        // Set contract owner using the initializer deployer
        self.ownable._owner.set(msg::sender());

        // Set NFT price
        self.price.set(price);

        // Set NFT token contract
        self.nft_token.set(nft_token);

        // Set agregators info
        if names.len() != agregators.len() || names.len() != tokens.len() {
            return Err(MarketError::MismatchData(MismatchData {}));
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

        // Change contract state to already initialized
        self.init.is_init.set(true);

        Ok(())
    }

    pub fn buy(&mut self, buyer: Address, name: FixedBytes<32>) -> Result<(), MarketError> {
        let payment_token = IERC20::new(self.price_feeds.get(name).token.get());
        let oracle = AggregatorV3Interface::new(self.price_feeds.get(name).agregator_address.get());

        // Get latest answer price
        let latest_answer = match oracle.latest_round_data(Call::new_in(self)) {
            Ok(data) => U256::from_limbs(data.1.into_limbs()),
            Err(_) => {
                return Err(MarketError::CallError(CallError { id: U256::from(2) }));
            }
        };

        let oracle_decimals = match oracle.decimals(Call::new_in(self)) {
            Ok(data) => U256::from(data),
            Err(_) => {
                return Err(MarketError::CallError(CallError { id: U256::from(3) }));
            }
        };

        let payment_decimals = match payment_token.decimals(Call::new_in(self)) {
            Ok(data) => U256::from(data),
            Err(_) => {
                return Err(MarketError::CallError(CallError { id: U256::from(4) }));
            }
        };

        // Scaled price
        let usd_price = self.price.get()
            * U256::from(10).pow(payment_decimals)
            * U256::from(10).pow(oracle_decimals);

        let amount_need = usd_price.div_ceil(latest_answer);

        let success = match payment_token.transfer_from(
            Call::new_in(self),
            buyer,
            contract::address(),
            amount_need,
        ) {
            Ok(data) => data,
            Err(_) => {
                return Err(MarketError::CallError(CallError { id: U256::from(5) }));
            }
        };

        if !success {
            return Err(MarketError::NotBought(NotBought {}).into());
        }

        // Increasing the total collected by this payment token
        let collected = self.total_collected.get(payment_token.address).clone();
        self.total_collected
            .setter(payment_token.address)
            .set(collected + amount_need);

        let seabrick = ISeabrick::new(self.nft_token.get());

        let new_id = match seabrick.total_supply(Call::new_in(self)) {
            Ok(data) => data,
            Err(_) => {
                return Err(MarketError::CallError(CallError { id: U256::from(6) }));
            }
        };

        if let Err(_) = seabrick.mint(Call::new_in(self), buyer) {
            return Err(MarketError::CallError(CallError { id: U256::from(7) }));
        }

        evm::log(NFTSold {
            owner: buyer,
            id: new_id,
        });

        Ok(())
    }

    pub fn claim(&mut self, name: FixedBytes<32>) -> Result<(), MarketError> {
        if let Err(_) = self.ownable.only_owner() {
            return Err(MarketError::CallError(CallError { id: U256::from(1) }));
        }

        let claim_token = IERC20::new(self.price_feeds.get(name).token.get());
        let collected = self.total_collected.get(claim_token.address).clone();

        // Decreasing the total collected by this payment token to ZERO since everything was claimed
        self.total_collected
            .setter(claim_token.address)
            .set(U256::ZERO);

        let success = match claim_token.transfer(Call::new_in(self), msg::sender(), collected) {
            Ok(data) => data,
            Err(_) => {
                return Err(MarketError::CallError(CallError { id: U256::from(2) }));
            }
        };

        if !success {
            return Err(MarketError::NotClaimed(NotClaimed {}));
        }

        Ok(())
    }
}
