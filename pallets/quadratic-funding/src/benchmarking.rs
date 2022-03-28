//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as QuadraticFunding;
use codec::alloc::string::ToString;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use primitives::currency::CurrencyId;

benchmarks! {
    start_round {
        let s in 1 .. 60u32;
    }: _(RawOrigin::Root, s.into(), CurrencyId::DORA, "dora".to_string().into())
    verify {
        assert!(Rounds::<T>::contains_key(&s));
    }

}

impl_benchmark_test_suite!(
    QuadraticFunding,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
