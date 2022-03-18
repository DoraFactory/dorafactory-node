#![allow(clippy::from_over_into)]
use codec::{Decode, Encode, MaxEncodedLen};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
    // Relay chain token.
    ROC,
    // Native TokenSymbol
    DORA,
    // Parachain B token
    // FF,
    // Demo Parachain token
    DD,
}
