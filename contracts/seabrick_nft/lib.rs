#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;
extern crate erc721;
extern crate initialization;
extern crate ownable;

use alloc::{format, string::String, vec, vec::Vec};
use erc721::{ERC721Params, ERC721};
use initialization::Initialization;
use ownable::Ownable;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::{entrypoint, external, sol_storage},
};

pub struct SeabrickParams;

/// Immutable definitions
impl ERC721Params for SeabrickParams {
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
        ERC721<SeabrickParams> erc721;
        #[borrow]
        Ownable ownable;
        #[borrow]
        Initialization init;
        uint256 total_supply;
    }
}

#[external]
#[inherit(ERC721<SeabrickParams>, Ownable)]
impl Seabrick {
    pub fn initialization(&mut self, owner: Address) -> Result<(), Vec<u8>> {
        // Check if already init. Revert if already init
        self.init._check_init()?;

        // Set contract owner
        self.ownable._owner.set(owner);

        // Change contract state to already initialized
        self.init.is_init.set(true);

        Ok(())
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.erc721._burn(token_id)?;
        let supply = self.total_supply.get();
        self.total_supply.set(supply - U256::from(1));
        Ok(())
    }

    pub fn mint(&mut self, to: Address) -> Result<(), Vec<u8>> {
        self.ownable.only_owner()?;
        let next_id = self.total_supply.get();
        self.erc721._mint(to, next_id)?;
        self.total_supply.set(next_id + U256::from(1));
        Ok(())
    }
}
