use crate::{mock::*, Error, Event as QFEvent};
use frame_support::{assert_noop, assert_ok};
use primitives::currency::CurrencyId;
use serde::de::Unexpected::Option;
use sp_runtime::traits::{BlakeTwo256, Hash, UniqueSaturatedFrom};
use std::ops::Sub;
use std::ptr::hash;

#[test]
fn test_start_round_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            100
        ));
        assert!(QuadraticFunding::rounds(1).is_some());
    })
}

#[test]
fn test_donate_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            100
        ));
        assert_ok!(QuadraticFunding::donate(
            Origin::signed(1),
            1,
            100_000_000_000_000_000_000,
            CurrencyId::DORA,
        ));
        assert_eq!(
            QuadraticFunding::rounds(1).unwrap().pre_tax_support_pool,
            100_000_000_000_000_000_000
        );
    })
}

#[test]
fn test_register_project_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            100
        ));
        assert_ok!(QuadraticFunding::donate(
            Origin::signed(1),
            1,
            100_000_000_000_000_000_000,
            CurrencyId::DORA,
        ));
        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(2),
                1,
                project_hash,
                "project".to_string().into()
            ))
        );
        assert!(QuadraticFunding::projects(1, project_hash).is_some());
    })
}

#[test]
fn test_vote_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            100
        ));
        assert_ok!(QuadraticFunding::donate(
            Origin::signed(1),
            1,
            100_000_000_000_000_000_000,
            CurrencyId::DORA,
        ));
        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(2),
                1,
                project_hash,
                "project".to_string().into()
            ))
        );
        let ballot_count = 3;
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            1,
            project_hash,
            ballot_count
        ));
        assert_eq!(
            QuadraticFunding::projects(1, project_hash)
                .unwrap()
                .total_votes,
            ballot_count
        );
    })
}

#[test]
fn test_end_round_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            100
        ));
        assert_ok!(QuadraticFunding::donate(
            Origin::signed(1),
            1,
            100_000_000_000_000_000_000,
            CurrencyId::DORA,
        ));
        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(2),
                1,
                project_hash,
                "project".to_string().into()
            ))
        );
        let ballot_count = 3;
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            1,
            project_hash,
            ballot_count
        ));
        assert_ok!(QuadraticFunding::end_round(Origin::root(), 1));
        assert_eq!(QuadraticFunding::rounds(1).unwrap().ongoing, false);
    })
}
