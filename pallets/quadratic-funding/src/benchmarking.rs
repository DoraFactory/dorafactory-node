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

// donate {
//     let caller: T::AccountId = whitelisted_caller();
//     let _ = <T as pallet::Config>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
// }: _(RawOrigin::Signed(caller), BalanceOf::<T>::max_value())
//
// register_project {
//     let caller: T::AccountId = whitelisted_caller();
// }: _(RawOrigin::Signed(caller), 1, vec![b'X'; 256], vec![b'X'; 256])

// start_round {m
//     let s in 1 .. 60;
//     let caller: T::AccountId = whitelisted_caller();
//     let _ = <T as pallet::Config>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
//     let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller));
//     QuadraticFunding::<T>::fund(caller_origin.clone(), 1000000u32.into())?;
//     let mut project_indexes: Vec<u32> = Vec::new();
//     for i in 0 .. s {
//         QuadraticFunding::<T>::create_project(caller_origin.clone(), vec![b'X'; 256], vec![b'X'; 256], vec![b'X'; 256], vec![b'X'; 256])?;
//         project_indexes.push(i);
//     }
// }: _(RawOrigin::Root, 100u32.into(), 200u32.into(), 10u32.into(), project_indexes)
//
// cancel_round {
//     let caller: T::AccountId = whitelisted_caller();
//     let _ = <T as pallet::Config>::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
//     let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller));
//     QuadraticFunding::<T>::fund(caller_origin.clone(), 1000000u32.into())?;
//     QuadraticFunding::<T>::create_project(caller_origin.clone(), vec![b'X'; 256], vec![b'X'; 256], vec![b'X'; 256], vec![b'X'; 256])?;
//     QuadraticFunding::<T>::schedule_round(<T as frame_system::Config>::Origin::from(RawOrigin::Root), 100u32.into(), 200u32.into(), 10u32.into(), vec![0])?;
// }: _(RawOrigin::Root, 0)
//
// contribute {
//     let caller2: T::AccountId = whitelisted_caller();
//     let _ = <T as pallet::Config>::Currency::make_free_balance_be(&caller2, BalanceOf::<T>::max_value());
//     let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller2.clone()));
//     QuadraticFunding::<T>::fund(caller_origin.clone(), 1000000u32.into())?;
//     QuadraticFunding::<T>::create_project(caller_origin.clone(), vec![b'X'; 256], vec![b'X'; 256], vec![b'X'; 256], vec![b'X'; 256])?;
//     QuadraticFunding::<T>::schedule_round(<T as frame_system::Config>::Origin::from(RawOrigin::Root), 100u32.into(), 200u32.into(), 10u32.into(), vec![0])?;
//     frame_system::Pallet::<T>::set_block_number(150u32.into());
// }: _(RawOrigin::Signed(caller2), 0, 100u32.into())
//
// finalize_round {
//     let caller2: T::AccountId = whitelisted_caller();
//     let _ = <T as pallet::Config>::Currency::make_free_balance_be(&caller2, BalanceOf::<T>::max_value());
//     let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller2.clone()));
//     QuadraticFunding::<T>::fund(caller_origin.clone(), 1000000u32.into())?;
//     QuadraticFunding::<T>::create_project(caller_origin.clone(), vec![b'X'; 256], vec![b'X'; 256], vec![b'X'; 256], vec![b'X'; 256])?;
//     QuadraticFunding::<T>::schedule_round(<T as frame_system::Config>::Origin::from(RawOrigin::Root), 100u32.into(), 200u32.into(), 10u32.into(), vec![0])?;
//     frame_system::Pallet::<T>::set_block_number(150u32.into());
//     QuadraticFunding::<T>::contribute(caller_origin.clone(), 0, 100u32.into())?;
//     frame_system::Pallet::<T>::set_block_number(210u32.into());
// }: _(RawOrigin::Root, 0)
//
//
//
// donate {
//     let caller = account("caller", 0, 0);
//     let id in 1...1000
//
//             // let caller: T::AccountId = whitelisted_caller();
//
//
// }: _(RawOrigin::Signed(caller), id, amount, currencyid)
//
// do_something {
//     let s in 0 .. 100;
//     let caller: T::AccountId = whitelisted_caller();
// }: _(RawOrigin::Signed(caller), s)
// verify {
//     assert_eq!(Something::<T>::get(), Some(s));
// }

// impl_benchmark_test_suite!(QuadraticFunding, crate::mock::new_test_ext(), crate::mock::Test);

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::mock::{new_test_ext, Test};
//     use frame_support::assert_ok;
//
//     #[test]
//     fn test_benchmarks() {
//         new_test_ext().execute_with(|| {
//             assert_ok!(test_benchmark_start_round::<Test>());
//         });
//     }
// }
impl_benchmark_test_suite!(
    QuadraticFunding,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
