#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;
extern crate erc721;

use alloc::{format, string::String, vec, vec::Vec};
use erc721::{ERC721Params, ERC721};
use stylus_sdk::{
    alloy_primitives::U256,
    prelude::{entrypoint, external, sol_storage},
};

pub struct SeabrickParams;

/// Immutable definitions
impl ERC721Params for SeabrickParams {
    const NAME: &'static str = "SeaBrick NFT";
    const SYMBOL: &'static str = "SB_NFT";

    fn token_uri(token_id: U256) -> String {
        format!(
            "https://ipfs.filebase.io/ipfs/QmUGC8GPVq2s8TU2pQaBfR2WYM3MtferDtu6kwTb6GCWFx/{}",
            // This is temporary, just for the sake of the given ipfs link
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
        uint256 total_supply;
    }
}

#[external]
#[inherit(ERC721<SeabrickParams>)]
impl Seabrick {
    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.erc721._burn(token_id)?;
        let supply = self.total_supply.get();
        self.total_supply.set(supply - U256::from(1));
        Ok(())
    }
}
