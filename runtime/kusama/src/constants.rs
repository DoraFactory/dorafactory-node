pub mod time {
    use primitives::{BlockNumber};
    /// This determines the average expected block time that we are targeting.
    /// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
    /// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
    /// up by `pallet_aura` to implement `fn slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: u64 = 12000;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}

pub mod currency {
    #![allow(clippy::from_over_into)]
    use codec::{Decode, Encode, MaxEncodedLen};
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    use scale_info::TypeInfo;
    use sp_runtime::RuntimeDebug;
    use sp_std::prelude::*;
    use primitives::{Balance, DOLLARS, MILLICENTS};

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
        KSM,
        // Native TokenSymbol
        DORA,
        // Other Parachain token
    }

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 2 * DOLLARS + (bytes as Balance) * 30 * MILLICENTS
    }
}
