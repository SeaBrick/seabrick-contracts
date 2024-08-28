//! Ownable contract.
//! The logic was based off of: https://github.com/OpenZeppelin/rust-contracts-stylus/blob/main/contracts/src/access/ownable.rs

extern crate alloc;

use stylus_sdk::{
    alloy_sol_types::sol,
    prelude::{sol_storage, SolidityError},
};

sol! {
    /// The contract was already initialized
    #[derive(Debug)]
    #[allow(missing_docs)]
    error AlreadyInit();

}

#[derive(SolidityError, Debug)]
pub enum InitializationError {
    /// Contract already init
    AlreadyInitialized(AlreadyInit),
}

sol_storage! {
    pub struct Initialization {
        bool is_init;
    }
}

impl Initialization {
    pub fn _check_init(&mut self) -> Result<(), InitializationError> {
        let init_status = self.is_init.get();

        if init_status {
            return Err(InitializationError::AlreadyInitialized(AlreadyInit {}));
        }

        Ok(())
    }

    pub fn _set_init(&mut self, value: bool) {
        self.is_init.set(value)
    }
}
