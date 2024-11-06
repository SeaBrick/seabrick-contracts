#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;

mod erc721;
mod initialization;
mod ownable;

use alloc::{format, string::String, vec::Vec};
use alloy_sol_types::sol;
use erc721::{Erc721, Erc721Error, Erc721Params};
use initialization::{Initialization, InitializationError};
use ownable::Ownable;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    evm, msg,
    prelude::{entrypoint, public, sol_storage, SolidityError},
};

pub struct SeabrickParams;

/// Immutable definitions
impl Erc721Params for SeabrickParams {
    const NAME: &'static str = "SeaBrick NFT";
    const SYMBOL: &'static str = "SB_NFT";

    fn token_uri(token_id: U256) -> String {
        format!(
            "https://ipfs.filebase.io/ipfs/QmUGC8GPVq2s8TU2pQaBfR2WYM3MtferDtu6kwTb6GCWFx/{}.jpg",
            // FIXME: This is temporary, just for the sake of the given ipfs link
            // token_id
            token_id % U256::from(5) + U256::from(1)
        )
    }
}

sol_storage! {
    // Makes Seabrick the entrypoint
    #[entrypoint]
    pub struct Seabrick {
        #[borrow]
        Erc721<SeabrickParams> erc721;
        #[borrow]
        Ownable ownable;
        #[borrow]
        Initialization init;
        mapping(address => bool) minters;
    }
}

sol! {
    event MinterUpdated(address minter, bool status);

    error OnlyMinters();
}

#[derive(SolidityError)]
pub enum SeabrickError {
    OnlyMinters(OnlyMinters),
}

#[public]
#[inherit(Erc721<SeabrickParams>, Ownable)]
impl Seabrick {
    pub fn initialization(&mut self, owner: Address) -> Result<(), InitializationError> {
        // Check if already init. Revert if already init
        self.init._check_init()?;

        // Set contract owner
        self.ownable._transfer_ownership(owner);

        // Change contract state to already initialized
        self.init._set_init(true);

        Ok(())
    }

    pub fn set_minter(&mut self, minter: Address, status: bool) -> Result<(), Vec<u8>> {
        self.ownable.only_owner()?;
        self.minters.setter(minter).set(status);

        evm::log(MinterUpdated { minter, status });

        Ok(())
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Erc721Error> {
        self.erc721.burn(msg::sender(), token_id)?;
        Ok(())
    }

    pub fn mint(&mut self, to: Address) -> Result<U256, Vec<u8>> {
        if !self.minters.get(msg::sender()) {
            return Err(SeabrickError::OnlyMinters(OnlyMinters {}).into());
        }

        self.erc721.mint(to)?;
        Ok(self.erc721.total_supply.get() - U256::from(1u8))
    }

    pub fn mint_batch(&mut self, to: Address, amount: u8) -> Result<(), Vec<u8>> {
        if !self.minters.get(msg::sender()) {
            return Err(SeabrickError::OnlyMinters(OnlyMinters {}).into());
        }

        for _ in 0..amount.into() {
            self.erc721.mint(to)?;
        }

        Ok(())
    }
}
