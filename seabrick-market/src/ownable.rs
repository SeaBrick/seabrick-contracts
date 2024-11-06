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

    /// * `ownership_contract` - Address that's not allowed to become the ownership contract.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error InvalidOwnership(address ownership_contract);
}

#[derive(SolidityError, Debug)]
pub enum OwnableError {
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(OwnableUnauthorizedAccount),
    /// The ownership address is not a valid ownership contract
    InvalidOwnership(InvalidOwnership),
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

    pub fn change_ownership_contract(&mut self, new_address: Address) -> Result<(), Vec<u8>> {
        self.only_owner()?;

        // We check the owner on target contract to avoid losing the ownership
        let target_owner_address = Ownership::new(new_address).owner(Call::new())?;

        // If target ownership address is not a contract, it will fail the transaction
        // If target ownership address owner is not the same that the current one
        //    we fail the transaction to avoid losing the ownership of the contracts
        if target_owner_address != self.owner()? {
            return Err(OwnableError::InvalidOwnership(InvalidOwnership {
                ownership_contract: target_owner_address,
            })
            .into());
        }

        // Change the ownership contract address
        self._ownership.set(new_address);

        Ok(())
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
