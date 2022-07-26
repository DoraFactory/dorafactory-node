use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
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
            2
        ));
        assert!(QuadraticFunding::rounds(1).is_some());
    })
}

#[test]
fn test_start_round_must_be_root() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            QuadraticFunding::start_round(
                Origin::signed(1),
                1,
                CurrencyId::DORA,
                "doraRound".to_string().into(),
                1,
                2
            ),
            DispatchError::BadOrigin
        );
    })
}

#[test]
fn test_start_round_name_len() {
    new_test_ext().execute_with(|| {
        let round_name = "012345678901234567890123456789012";

        assert_noop!(
            QuadraticFunding::start_round(
                Origin::root(),
                1,
                CurrencyId::DORA,
                round_name.to_string().into(), // len(round_name) = 33, It's longer than 32.
                1,
                2
            ),
            Error::<Runtime>::BadMetadata
        );
    })
}

#[test]
fn test_start_round_can_not_repeat() {
    new_test_ext().execute_with(|| {
        let round_name = "doraRound";
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            round_name.to_string().into(),
            1,
            2
        ));
        assert_noop!(
            QuadraticFunding::start_round(
                Origin::root(),
                1,
                CurrencyId::DORA,
                round_name.to_string().into(), // The round name must be unique.
                1,
                2
            ),
            Error::<Runtime>::RoundExisted
        );
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
            2
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
fn test_donate_inexist_round() {
    new_test_ext().execute_with(|| {
        let inexist_round_id = 1;

        assert_noop!(
            QuadraticFunding::donate(
                Origin::signed(1),
                inexist_round_id, // Can't sponsor a round that doesn't exist.
                100_000_000_000_000_000_000,
                CurrencyId::DORA,
            ),
            Error::<Runtime>::RoundNotExist
        );
    })
}

#[test]
fn test_donate_ended_round() {
    new_test_ext().execute_with(|| {
        let round_id = 1;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));
        assert_ok!(QuadraticFunding::end_round(Origin::root(), round_id));

        assert_noop!(
            QuadraticFunding::donate(
                Origin::signed(1),
                round_id, // Can't sponsor a closed round.
                100_000_000_000_000_000_000,
                CurrencyId::DORA,
            ),
            Error::<Runtime>::RoundHasEnded
        );
    })
}

#[test]
fn test_donate_with_wrong_currencyid() {
    new_test_ext().execute_with(|| {
        let round_id = 1;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));
        assert_noop!(
            QuadraticFunding::donate(
                Origin::signed(1),
                round_id,
                100_000_000_000_000_000_000,
                CurrencyId::KSM, // Can't sponsor in unsupported currency.
            ),
            Error::<Runtime>::MismatchingCurencyId
        );
    })
}

#[test]
fn test_donate_with_too_small_money() {
    new_test_ext().execute_with(|| {
        let round_id = 1;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));
        assert_noop!(
            QuadraticFunding::donate(
                Origin::signed(1),
                round_id,
                1_000_000_000_000, // donate money must > min_unit_number, min_unit_number = NumberOfUnit * VoteUnit = 1 * DORA
                CurrencyId::DORA
            ),
            Error::<Runtime>::DonationTooSmall
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
            2
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
fn test_register_project_with_short_name() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));

        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_noop!(
            QuadraticFunding::register_project(
                Origin::signed(2),
                1,
                project_hash,
                "pr".to_string().into()
            ),
            Error::<Runtime>::ProjectNameTooShort
        );
    })
}

#[test]
fn test_register_project_with_longer_name() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));

        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        let project_name = "012345678901234567890123456789012"; // len(project_name) = 33

        assert_noop!(
            QuadraticFunding::register_project(
                Origin::signed(2),
                1,
                project_hash,
                project_name.to_string().into()
            ),
            Error::<Runtime>::ProjectNameTooLong
        );
    })
}

#[test]
fn test_register_project_inexist_round() {
    new_test_ext().execute_with(|| {
        let inexist_round_id = 1;
        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));

        assert_noop!(
            QuadraticFunding::register_project(
                Origin::signed(2),
                inexist_round_id,
                project_hash,
                "project".to_string().into()
            ),
            Error::<Runtime>::RoundNotExist
        );
    })
}

#[test]
fn test_register_project_ended_round() {
    new_test_ext().execute_with(|| {
        let round_id = 1;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));
        assert_ok!(QuadraticFunding::end_round(Origin::root(), round_id));

        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_noop!(
            QuadraticFunding::register_project(
                Origin::signed(2),
                round_id,
                project_hash,
                "project".to_string().into()
            ),
            Error::<Runtime>::RoundHasEnded
        );
    })
}

#[test]
fn test_register_project_with_same_name() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
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

        assert_noop!(
            QuadraticFunding::register_project(
                Origin::signed(3),
                1,
                project_hash,
                "project".to_string().into()
            ),
            Error::<Runtime>::DuplicateProject
        );
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
            2
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
fn test_vote_inexist_round() {
    new_test_ext().execute_with(|| {
        let inexist_round_id = 1;

        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        let ballot_count = 3;

        assert_noop!(
            QuadraticFunding::vote(
                Origin::signed(3),
                CurrencyId::DORA,
                inexist_round_id,
                project_hash,
                ballot_count
            ),
            Error::<Runtime>::RoundNotExist
        );
    })
}

#[test]
fn test_vote_ended_round() {
    new_test_ext().execute_with(|| {
        let round_id = 1;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));
        assert_ok!(QuadraticFunding::end_round(Origin::root(), round_id));

        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        let ballot_count = 3;

        assert_noop!(
            QuadraticFunding::vote(
                Origin::signed(3),
                CurrencyId::DORA,
                round_id,
                project_hash,
                ballot_count
            ),
            Error::<Runtime>::RoundHasEnded
        );
    })
}

#[test]
fn test_vote_inexist_project() {
    new_test_ext().execute_with(|| {
        let round_id = 1;

        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));

        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        let ballot_count = 3;

        assert_noop!(
            QuadraticFunding::vote(
                Origin::signed(3),
                CurrencyId::DORA,
                1,
                project_hash,
                ballot_count
            ),
            Error::<Runtime>::ProjectNotExist
        );
    })
}

#[test]
fn test_vote_with_wrong_currencyid() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
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

        assert_noop!(
            QuadraticFunding::vote(
                Origin::signed(3),
                CurrencyId::KSM,
                1,
                project_hash,
                ballot_count
            ),
            Error::<Runtime>::MismatchingCurencyId
        );
    })
}

#[test]
fn test_vote_with_zero_ballot() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
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

        let ballot_count = 0;

        assert_noop!(
            QuadraticFunding::vote(
                Origin::signed(3),
                CurrencyId::DORA,
                1,
                project_hash,
                ballot_count
            ),
            Error::<Runtime>::InvalidBallot
        );
    })
}

#[test]
fn test_vote_must_reserve() {
    new_test_ext().execute_with(|| {
        let multi_reserve = 2;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            multi_reserve // must_reserve_amount = multi_reserve * ReserveUnit = 2 DORA
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

        assert_noop!(
            QuadraticFunding::vote(
                Origin::signed(4), // must_reserve_amount = 2 DORA, balance_of(4) = 1 DORA, The balance of account 4 is not enough for reserve.
                CurrencyId::DORA,
                1,
                project_hash,
                ballot_count
            ),
            Error::<Runtime>::InsufficientReserveDora
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
            2
        ));

        assert_ok!(QuadraticFunding::end_round(Origin::root(), 1));
        assert_eq!(QuadraticFunding::rounds(1).unwrap().ongoing, false);
    })
}

#[test]
fn test_end_inexist_round() {
    new_test_ext().execute_with(|| {
        let inexist_round_id = 1;

        assert_noop!(
            QuadraticFunding::end_round(Origin::root(), inexist_round_id),
            Error::<Runtime>::RoundNotExist
        );
    })
}

#[test]
fn test_end_ended_round() {
    new_test_ext().execute_with(|| {
        let round_id = 1;

        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));
        assert_ok!(QuadraticFunding::end_round(Origin::root(), round_id));

        assert_noop!(
            QuadraticFunding::end_round(Origin::root(), round_id),
            Error::<Runtime>::RoundHasEnded
        );
    })
}

#[test]
fn test_end_round_must_be_root() {
    new_test_ext().execute_with(|| {
        let round_id = 1;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));

        assert_noop!(
            QuadraticFunding::end_round(Origin::signed(1), round_id),
            DispatchError::BadOrigin
        );
    })
}

#[test]
fn test_a_complete_round_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
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
