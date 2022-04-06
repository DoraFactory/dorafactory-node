//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as QuadraticFunding;
use codec::alloc::string::ToString;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use hex_literal::hex;
use primitives::currency::CurrencyId;
use sp_runtime::traits::{AccountIdConversion, Hash, UniqueSaturatedFrom};

const SEED: u32 = 0;

benchmarks! {


}

impl_benchmark_test_suite!(
    QuadraticFunding,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
