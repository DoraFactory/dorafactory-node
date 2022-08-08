use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError, PalletId};
use primitives::{currency::CurrencyId, DOLLARS};
use serde::de::Unexpected::Option;
use sp_runtime::traits::{
    AccountIdConversion, BlakeTwo256, Hash, IdentifyAccount, UniqueSaturatedFrom,
};
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
fn test_start_round_with_short_name() {
    new_test_ext().execute_with(|| {
        let round_name = "01";

        assert_noop!(
            QuadraticFunding::start_round(
                Origin::root(),
                1,
                CurrencyId::DORA,
                round_name.to_string().into(), // len(round_name) = 2, It's shorter than 32.
                1,
                2
            ),
            Error::<Runtime>::RoundNameTooShort
        );
    })
}

#[test]
fn test_start_round_with_long_name() {
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
            Error::<Runtime>::RoundNameTooLong
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
        let round_id = 1;
        let donate_amount = 1_000_000_000_000_000;

        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            5,
            2
        ));
        assert_ok!(QuadraticFunding::donate(
            Origin::signed(1),
            round_id,
            donate_amount,
            CurrencyId::DORA,
        ));
        assert_eq!(
            QuadraticFunding::rounds(1).unwrap().pre_tax_support_pool,
            donate_amount
        );

        // donate amount in admin account
        assert_eq!(
            Balances::free_balance(QuadraticFunding::round_admin_account(round_id)),
            donate_amount - 6u128.checked_mul(donate_amount / 1000).unwrap()
        );
        // fee amount in pallet account
        assert_eq!(
            Balances::free_balance(QuadraticFunding::account_id()),
            6u128.checked_mul(donate_amount / 1000).unwrap()
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
                1_000_000_000_000_000,
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
                1_000_000_000_000_000,
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
                1_000_000_000_000_000,
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
fn test_register_project_in_diff_round_with_same_hash() {
    // 同样的hash可以参加不同的round
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound_1".to_string().into(),
            1,
            2
        ));
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            2,
            CurrencyId::DORA,
            "doraRound_2".to_string().into(),
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
        assert_ok!(QuadraticFunding::register_project(
            Origin::signed(3),
            2,
            project_hash,
            "project".to_string().into()
        ));

        assert!(QuadraticFunding::projects(1, project_hash).is_some());
        assert!(QuadraticFunding::projects(2, project_hash).is_some());
    })
}

#[test]
fn test_vote_works() {
    new_test_ext().execute_with(|| {
        let round_id = 1;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            5,
            2
        ));
        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(2),
                round_id,
                project_hash,
                "project".to_string().into()
            ))
        );
        let ballot_count = 3;
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            round_id,
            project_hash,
            ballot_count
        ));
        assert_eq!(
            QuadraticFunding::projects(round_id, project_hash)
                .unwrap()
                .total_votes,
            ballot_count
        );
        // now_free_balance(3) = balance_of(3) - vote_token - reserve_token
        assert_eq!(
            Balances::free_balance(3),
            100 * DOLLARS - (1 + 2 + 3) * 1_000_000_000_000 - 2 * 1_000_000_000_000
        );
        // vote amount in admin account
        assert_eq!(
            Balances::free_balance(QuadraticFunding::round_admin_account(round_id)),
            (1 + 2 + 3) * 1_000_000_000_000 - QuadraticFunding::cal_amount((1 + 2 + 3), true)
        );
        // fee amount in pallet account
        assert_eq!(
            Balances::free_balance(QuadraticFunding::account_id()),
            QuadraticFunding::cal_amount((1 + 2 + 3), true)
        );
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            round_id,
            project_hash,
            1
        ));
        assert_eq!(
            Balances::free_balance(3),
            100 * DOLLARS - (1 + 2 + 3 + 4) * 1_000_000_000_000 - 2 * 1_000_000_000_000
        );
        // vote amount in admin account
        assert_eq!(
            Balances::free_balance(QuadraticFunding::round_admin_account(round_id)),
            (1 + 2 + 3 + 4) * 1_000_000_000_000
                - QuadraticFunding::cal_amount((1 + 2 + 3 + 4), true)
        );
    })
}

#[test]
fn test_vote_can_not_reserve_again_in_same_round() {
    new_test_ext().execute_with(|| {
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            1,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            1,
            2
        ));
        let project_hash_0 = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(2),
                1,
                project_hash_0,
                "project".to_string().into()
            ))
        );
        let project_hash_1 = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 1u128));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(4),
                1,
                project_hash_1,
                "project".to_string().into()
            ))
        );
        let ballot_count = 3;
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            1,
            project_hash_0,
            ballot_count
        ));

        // first_free_balance(3) = balance_of(3) - vote_token - reserve_token
        assert_eq!(
            Balances::free_balance(3),
            100 * DOLLARS - (1 + 2 + 3) * 1_000_000_000_000 - 2 * 1_000_000_000_000
        );
        // in same round only uses reserve once.
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            1,
            project_hash_1,
            ballot_count
        ));
        // second_free_balance(3) = balance_of(3) - vote_token_0 - reserve_token - vote_token_1
        assert_eq!(
            Balances::free_balance(3),
            100 * DOLLARS
                - (1 + 2 + 3) * 1_000_000_000_000
                - 2 * 1_000_000_000_000
                - (1 + 2 + 3) * 1_000_000_000_000
        );
    })
}

#[test]
fn test_vote_need_reserve_again_in_diff_round() {
    new_test_ext().execute_with(|| {
        let first_round_id = 1;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            first_round_id,
            CurrencyId::DORA,
            "doraRound1".to_string().into(),
            1,
            2
        ));
        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(2),
                first_round_id,
                project_hash,
                "project".to_string().into()
            ))
        );
        let ballot_count = 3;
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            first_round_id,
            project_hash,
            ballot_count
        ));
        // first_free_balance(3) = balance_of(3) - vote_token - reserve_token
        assert_eq!(
            Balances::free_balance(3),
            100 * DOLLARS - (1 + 2 + 3) * 1_000_000_000_000 - 2 * 1_000_000_000_000
        );

        let second_round_id = 2;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            second_round_id,
            CurrencyId::DORA,
            "doraRound2".to_string().into(),
            1,
            2
        ));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(2),
                second_round_id,
                project_hash,
                "project".to_string().into()
            ))
        );
        //  The first vote of each round must be reserved.
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            second_round_id,
            project_hash,
            ballot_count
        ));

        // second_free_balance(3) = balance_of(3) - vote_token_0 - reserve_token - vote_token_1
        assert_eq!(
            Balances::free_balance(3),
            100 * DOLLARS
                - (1 + 2 + 3) * 1_000_000_000_000
                - 2 * 1_000_000_000_000
                - (1 + 2 + 3) * 1_000_000_000_000
                - 2 * 1_000_000_000_000
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
        let round_id = 1;
        let donate_amount = 1_000_000_000_000_000;
        assert_ok!(QuadraticFunding::start_round(
            Origin::root(),
            round_id,
            CurrencyId::DORA,
            "doraRound".to_string().into(),
            5,
            2
        ));
        assert_ok!(QuadraticFunding::donate(
            Origin::signed(1),
            round_id,
            donate_amount,
            CurrencyId::DORA,
        ));
        let project_hash = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.dot"), 0u128));
        assert_ok!(
            (QuadraticFunding::register_project(
                Origin::signed(1),
                round_id,
                project_hash,
                "project".to_string().into()
            ))
        );
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(2),
            CurrencyId::DORA,
            round_id,
            project_hash,
            2
        ));
        assert_ok!(QuadraticFunding::vote(
            Origin::signed(3),
            CurrencyId::DORA,
            round_id,
            project_hash,
            3
        ));
        assert_ok!(QuadraticFunding::end_round(Origin::root(), round_id));
        assert_eq!(QuadraticFunding::rounds(round_id).unwrap().ongoing, false);
        assert_eq!(
            Balances::free_balance(QuadraticFunding::round_admin_account(round_id)),
            donate_amount - 6u128.checked_mul(donate_amount / 1000).unwrap()
                + (1 + 2 + 3) * 1_000_000_000_000
                - QuadraticFunding::cal_amount(1 + 2 + 3, true)
                + (1 + 2) * 1_000_000_000_000
                - QuadraticFunding::cal_amount(1 + 2, true)
        );
    })
}
