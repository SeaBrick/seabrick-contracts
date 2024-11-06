#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;
mod initialization;

use initialization::{Initialization, InitializationError};
use stylus_sdk::{
    alloy_primitives::Address,
    alloy_sol_types::sol,
    evm, msg,
    prelude::{entrypoint, public, sol_storage, SolidityError},
};

sol! {
    /// Emitted when ownership gets transferred between accounts.
    #[allow(missing_docs)]
    event OwnershipTransferred(address indexed previous_owner, address indexed new_owner);
}

sol! {
    /// The caller account is not authorized to perform an operation.
    ///
    /// * `account` - Account that was found to not be authorized.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error OwnershipUnauthorizedAccount(address account);
    /// The owner is not a valid owner account. (eg. `Address::ZERO`)
    ///
    /// * `owner` - Account that's not allowed to become the owner.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error OwnershipInvalidOwner(address owner);
}

#[derive(SolidityError, Debug)]
pub enum OwnershipError {
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(OwnershipUnauthorizedAccount),
    /// The owner is not a valid owner account. (eg. `Address::ZERO`)
    InvalidOwner(OwnershipInvalidOwner),
}

sol_storage! {
    #[entrypoint]
    pub struct Ownership {
        address _owner;
        #[borrow]
        Initialization init;
    }
}

impl Ownership {
    /// Checks if the [`msg::sender`] is set as the owner.
    ///
    /// # Errors
    ///
    /// If called by any account other than the owner, then the error
    /// [`Error::UnauthorizedAccount`] is returned.
    pub fn only_owner(&self) -> Result<(), OwnershipError> {
        let account = msg::sender();
        if self.owner() != account {
            return Err(OwnershipError::UnauthorizedAccount(
                OwnershipUnauthorizedAccount { account },
            ));
        }

        Ok(())
    }

    /// Transfers ownership of the contract to a new account (`new_owner`).
    /// Internal function without access restriction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - Account that's gonna be the next owner.
    pub fn _transfer_ownership(&mut self, new_owner: Address) {
        let previous_owner = self._owner.get();
        self._owner.set(new_owner);
        evm::log(OwnershipTransferred {
            previous_owner,
            new_owner,
        });
    }
}

#[public]
impl Ownership {
    pub fn initialization(&mut self, owner: Address) -> Result<(), InitializationError> {
        // Check if already init. Revert if already init
        self.init._check_init()?;

        // Set contract owner
        self._transfer_ownership(owner);

        // Change contract state to already initialized
        self.init._set_init(true);

        Ok(())
    }

    /// Returns the address of the current owner.
    pub fn owner(&self) -> Address {
        self._owner.get()
    }

    /// Transfers ownership of the contract to a new account (`new_owner`). Can
    /// only be called by the current owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - The next owner of this contract.
    ///
    /// # Errors
    ///
    /// If `new_owner` is the zero address, then the error
    /// [`OwnershipInvalidOwner`] is returned.
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), OwnershipError> {
        self.only_owner()?;

        if new_owner == Address::ZERO {
            return Err(OwnershipError::InvalidOwner(OwnershipInvalidOwner {
                owner: Address::ZERO,
            }));
        }

        self._transfer_ownership(new_owner);

        Ok(())
    }
}
