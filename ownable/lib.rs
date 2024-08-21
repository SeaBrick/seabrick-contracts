//! Ownable contract.
//! The logic was based off of: https://github.com/OpenZeppelin/rust-contracts-stylus/blob/main/contracts/src/access/ownable.rs

#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;

use stylus_sdk::{
    alloy_primitives::Address,
    alloy_sol_types::sol,
    evm, msg,
    prelude::{external, sol_storage, SolidityError},
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
    error OwnableUnauthorizedAccount(address account);
    /// The owner is not a valid owner account. (eg. `Address::ZERO`)
    ///
    /// * `owner` - Account that's not allowed to become the owner.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error OwnableInvalidOwner(address owner);
}

#[derive(SolidityError, Debug)]
pub enum Error {
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(OwnableUnauthorizedAccount),
    /// The owner is not a valid owner account. (eg. `Address::ZERO`)
    InvalidOwner(OwnableInvalidOwner),
}

sol_storage! {
    pub struct Ownable {
        address _owner;
    }
}

#[external]
impl Ownable {
    /// Returns the address of the current owner.
    pub fn owner(&self) -> Address {
        self._owner.get()
    }

    /// Checks if the [`msg::sender`] is set as the owner.
    ///
    /// # Errors
    ///
    /// If called by any account other than the owner, then the error
    /// [`Error::UnauthorizedAccount`] is returned.
    pub fn only_owner(&self) -> Result<(), Error> {
        let account = msg::sender();
        if self.owner() != account {
            return Err(Error::UnauthorizedAccount(OwnableUnauthorizedAccount {
                account,
            }));
        }

        Ok(())
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
    /// [`OwnableInvalidOwner`] is returned.
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Error> {
        self.only_owner()?;

        if new_owner == Address::ZERO {
            return Err(Error::InvalidOwner(OwnableInvalidOwner {
                owner: Address::ZERO,
            }));
        }

        self._transfer_ownership(new_owner);

        Ok(())
    }

    /// Leaves the contract without owner. It will not be possible to call
    /// [`Self::only_owner`] functions. Can only be called by the current owner.
    ///
    /// NOTE: Renouncing ownership will leave the contract without an owner,
    /// thereby disabling any functionality that is only available to the owner.
    ///
    /// # Errors
    ///
    /// If not called by the owner, then the error
    /// [`Error::UnauthorizedAccount`] is returned.
    pub fn renounce_ownership(&mut self) -> Result<(), Error> {
        self.only_owner()?;
        self._transfer_ownership(Address::ZERO);
        Ok(())
    }
}

impl Ownable {
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
