
//! Autogenerated weights for `pallet_qf`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-28, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// pallet_qf
// --extrinsic
// start_round
// --steps
// 20
// --repeat
// 10
// --output
// ./pallets/quadratic-funding/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_qf`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_qf::WeightInfo for WeightInfo<T> {
	// Storage: QuadraticFunding Rounds (r:1 w:1)
	fn start_round(_s: u32, ) -> Weight {
		(10_048_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}