//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as QuadraticFunding;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};

use frame_system::RawOrigin;
use primitives::currency::CurrencyId;

benchmarks! {
    start_round {
        let s in 1 .. 60u32;
        // let caller: T::AccountId = whitelisted_caller();
        // let _ = <T as pallet::Config>::MultiCurrency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        // let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller));
    }: _(RawOrigin::Root, s.into(), CurrencyId::DORA, vec![b'X'; 256])
    verify {
        assert!(Rounds::<T>::contains_key(&s));
    }
}

impl_benchmark_test_suite!(
    QuadraticFunding,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
