#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;

mod erc721;
mod initialization;
mod ownable;

use alloc::{format, string::String, vec};
use alloy_sol_types::sol;
use erc721::{Erc721, Erc721Params};
use initialization::Initialization;
use ownable::Ownable;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    msg,
    prelude::{entrypoint, external, sol_storage, SolidityError},
};

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

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

sol! {
    /// NFT not mint
    error NotMinted();

    /// NFT not Burn
    error NotBurned();

    error OnlyContractOwner();

    /// Error from a call
    error AlreadyInit();
}

#[derive(SolidityError)]
pub enum TokenError {
    NotMinted(NotMinted),
    NotBurned(NotBurned),
    OnlyContractOwner(OnlyContractOwner),
    AlreadyInit(AlreadyInit),
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
    }
}

#[external]
#[inherit(Erc721<SeabrickParams>, Ownable)]
impl Seabrick {
    pub fn initialization(&mut self, owner: Address) -> Result<(), TokenError> {
        // Check if already init. Revert if already init
        if let Err(_) = self.init._check_init() {
            return Err(TokenError::AlreadyInit(AlreadyInit {}));
        }

        // Set contract owner
        self.ownable._owner.set(owner);

        // Change contract state to already initialized
        self.init.is_init.set(true);

        Ok(())
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), TokenError> {
        if let Err(_) = self.erc721.burn(msg::sender(), token_id) {
            return Err(TokenError::NotBurned(NotBurned {}));
        }

        Ok(())
    }

    pub fn mint(&mut self, to: Address) -> Result<(), TokenError> {
        // if let Err(_) = self.ownable.only_owner() {
        //     return Err(TokenError::OnlyContractOwner(OnlyContractOwner {}));
        // }

        if let Err(_) = self.erc721.mint(to) {
            return Err(TokenError::NotMinted(NotMinted {}));
        }
        Ok(())
    }
}
