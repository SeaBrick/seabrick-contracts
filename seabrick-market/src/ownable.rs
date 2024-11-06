//! Ownable contract.

extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::{
    alloy_primitives::Address,
    alloy_sol_types::sol,
    call::Call,
    msg,
    prelude::{public, sol_interface, sol_storage, SolidityError},
};

sol_interface! {
    interface Ownership {
        function owner() external view returns (address);
    }
}

sol! {
    /// The caller account is not authorized to perform an operation.
    ///
    /// * `account` - Account that was found to not be authorized.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error OwnableUnauthorizedAccount(address account);
}

#[derive(SolidityError, Debug)]
pub enum OwnableError {
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(OwnableUnauthorizedAccount),
}

sol_storage! {
    pub struct Ownable {
        // Ownership contract address
        address _ownership;
    }
}

#[public]
impl Ownable {
    /// Returns the address of the current owner.
    pub fn owner(&self) -> Result<Address, Vec<u8>> {
        let owner_address = Ownership::new(self._ownership.get()).owner(Call::new())?;
        Ok(owner_address)
    }
}

impl Ownable {
    pub fn set_ownership_contract(&mut self, address: Address) {
        self._ownership.set(address)
    }

    /// Checks if the [`msg::sender`] is set as the owner.
    ///
    /// # Errors
    ///
    /// If called by any account other than the owner, then the error
    /// [`Error::UnauthorizedAccount`] is returned.
    pub fn only_owner(&self) -> Result<(), Vec<u8>> {
        let account = msg::sender();
        if self.owner()? != account {
            return Err(
                OwnableError::UnauthorizedAccount(OwnableUnauthorizedAccount { account }).into(),
            );
        }
        Ok(())
    }
}
