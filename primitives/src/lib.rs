#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::upper_case_acronyms)]
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature, RuntimeDebug,
};

pub use sp_runtime::{MultiAddress, Perbill};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

// A few exports that help ease life for downstream crates.
pub use frame_support::weights::{constants::WEIGHT_PER_SECOND, Weight, WeightToFeeCoefficient};

pub mod currency;

pub use crate::currency::CurrencyId;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Amount of an account.
pub type Amount = i128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

/// Money setting
// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLIUNIT: Balance = 1_000_000_000;
pub const MICROUNIT: Balance = 1_000_000;

pub const MILLICENTS: Balance = 1_000 * MICROUNIT;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS; // TODO: change DOLLARS price

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

#[derive(
    Encode,
    Decode,
    Eq,
    PartialEq,
    Copy,
    Clone,
    RuntimeDebug,
    PartialOrd,
    Ord,
    MaxEncodedLen,
    TypeInfo,
)]
#[repr(u8)]
pub enum ReserveIdentifier {
    CollatorSelection,
    Honzon,
    TransactionPayment,
    TransactionPaymentDeposit,

    // always the last, indicate number of variants
    Count,
}
