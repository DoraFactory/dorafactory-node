//! Autogenerated weights for pallet_dora_rewards
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-30, STEPS: `50`, REPEAT: 200, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/dorafactory-node
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_dora_rewards
// --extrinsic
// *
// --steps
// 50
// --repeat
// 200
// --template
// ./.maintain/frame-weight-template.hbs
// --output
// ./pallets/dora-rewards/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_dora_rewards.
pub trait WeightInfo {
    fn initialize_contributors_list() -> Weight;
    fn complete_initialization() -> Weight;
    fn claim_rewards() -> Weight;
}

/// Weights for pallet_dora_rewards using the Dora node and recommended hardware.
pub struct DoraWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for DoraWeight<T> {
    // Storage: DoraRewards InitBlock (r:1 w:0)
    // Storage: DoraRewards ContributorsInfo (r:0 w:2)
    fn initialize_contributors_list() -> Weight {
        // (39_174_000 as Weight)
        Weight::from_ref_time(39_174_000 as u64)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    // Storage: DoraRewards InitBlock (r:1 w:0)
    // Storage: DoraRewards EndBlock (r:0 w:1)
    fn complete_initialization() -> Weight {
        Weight::from_ref_time(18_768_000 as u64)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    // Storage: DoraRewards ContributorsInfo (r:1 w:1)
    // Storage: DoraRewards EndBlock (r:1 w:0)
    // Storage: DoraRewards InitBlock (r:1 w:0)
    // Storage: ParachainSystem ValidationData (r:1 w:0)
    // Storage: System Account (r:2 w:2)
    fn claim_rewards() -> Weight {
        // (72_147_000 as Weight)
        Weight::from_ref_time(72_147_000 as u64)
            .saturating_add(T::DbWeight::get().reads(6 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn initialize_contributors_list() -> Weight {
        // (39_174_000 as Weight)
        Weight::from_ref_time(39_174_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    fn complete_initialization() -> Weight {
        // (18_768_000 as Weight)
        Weight::from_ref_time(18_768_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn claim_rewards() -> Weight {
        // (72_147_000 as Weight)
        Weight::from_ref_time(72_147_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(6 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
    }
}
