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
    start_round {
    }: _(RawOrigin::Root, 1u32, CurrencyId::DORA, "dora".to_string().into())

    donate {
        QuadraticFunding::<T>::start_round(<T as frame_system::Config>::Origin::from(RawOrigin::Root), 1u32, CurrencyId::DORA, "dora".to_string().into());

        let alice: T::AccountId = account("alice", 0, SEED);
        let bob: T::AccountId = account("bob", 0, SEED);
        let token_amount = BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128);
    }: _(RawOrigin::Signed(alice), 1u32, token_amount, CurrencyId::DORA)

    register_project {
        QuadraticFunding::<T>::start_round(<T as frame_system::Config>::Origin::from(RawOrigin::Root), 1u32, CurrencyId::DORA, "dora".to_string().into());

        let alice: T::AccountId = account("alice", 0, SEED);
        let bob: T::AccountId = account("bob", 0, SEED);
        let token_amount = BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128);

        QuadraticFunding::<T>::donate(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u32, token_amount, CurrencyId::DORA);

        let project_hash = T::Hashing::hash_of(&1);
    }: _(RawOrigin::Signed(bob), 1u32, project_hash, "hack".to_string().into())

    vote {
        QuadraticFunding::<T>::start_round(<T as frame_system::Config>::Origin::from(RawOrigin::Root), 1u32, CurrencyId::DORA, "dora".to_string().into());

        let alice: T::AccountId = account("alice", 0, SEED);
        let bob: T::AccountId = account("bob", 0, SEED);
        let voter: T::AccountId = account("charlie", 0, SEED);
        let token_amount = BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128);

        QuadraticFunding::<T>::donate(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u32, token_amount, CurrencyId::DORA);

        let project_hash = T::Hashing::hash_of(&1);

        QuadraticFunding::<T>::register_project(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(bob)), 1u32, project_hash, "hack".to_string().into());
    }: _(RawOrigin::Signed(voter), CurrencyId::DORA, 1u32, project_hash, 12)

    end_round {
        QuadraticFunding::<T>::start_round(<T as frame_system::Config>::Origin::from(RawOrigin::Root), 1u32, CurrencyId::DORA, "dora".to_string().into());

        let alice: T::AccountId = account("alice", 0, SEED);
        let bob: T::AccountId = account("bob", 0, SEED);
        let voter: T::AccountId = account("charlie", 0, SEED);
        let token_amount = BalanceOf::<T>::unique_saturated_from(100_000_000_000_000u128);

        QuadraticFunding::<T>::donate(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(alice)), 1u32, token_amount, CurrencyId::DORA);

        let project_hash = T::Hashing::hash_of(&1);

        QuadraticFunding::<T>::register_project(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(bob)), 1u32, project_hash, "hack".to_string().into());
        QuadraticFunding::<T>::vote(<T as frame_system::Config>::Origin::from(RawOrigin::Signed(voter)), CurrencyId::DORA, 1u32, project_hash, 12);
    }: _(RawOrigin::Root, 1u32)
}

impl_benchmark_test_suite!(
    QuadraticFunding,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
