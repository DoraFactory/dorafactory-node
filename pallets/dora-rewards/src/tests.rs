// Copyright 2019-2021 DoraFactory Inc.
// This file is part of DoraFactory-KSM-parachain.

// DoraFactory-KSM-parachain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// DoraFactory-KSM-parachain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with DoraFactory-KSM-parachain.  If not, see <http://www.gnu.org/licenses/>.

//! Unit testing

use crate::*;
use frame_support::dispatch::DispatchError;
use frame_support::{assert_noop, assert_ok};
use mock::*;

// Constant that reflects the desired vesting period for the tests
// which is the lease period.
const VESTING: u32 = 8;

/// test the dora reward pallet's genesis build and imput the contributor correctly
#[test]
fn init_and_complete_contributor_with_sudo_correctly() {
    empty().execute_with(|| {
        // init lease block
        let init_block = DoraRewards::init_vesting_block();
        assert_ok!(
            // initialize the contributor list
            DoraRewards::initialize_contributors_list(
                Origin::root(),
                vec![(1, 100u32.into()), (2, 200u32.into()), (3, 300u32.into())]
            )
        );
        assert_ok!(DoraRewards::complete_initialization(
            Origin::root(),
            init_block + VESTING
        ));

        // check the contributors number
        assert_eq!(DoraRewards::total_contributors(), 3);

        // ensure the contributor exist
        assert!(DoraRewards::rewards_info(&1).is_some());
        assert!(DoraRewards::rewards_info(&1).is_some());
        assert!(DoraRewards::rewards_info(&1).is_some());
    })
}

/// initialize contributor with common account, but not sudo
/// this will dispatch a error with `BadOrigin`
#[test]
fn init_contributor_with_common_user() {
    empty().execute_with(|| {
        assert_noop!(
            DoraRewards::initialize_contributors_list(
                Origin::signed(0),
                vec![(1, 100u32.into()), (2, 200u32.into()), (3, 300u32.into())]
            ),
            DispatchError::BadOrigin
        );
    });
}

// #[test]
// fn initialize_with_no_contributor() {
//     empty().execute_with(|| {
//         // init lease block
//         let init_block = DoraRewards::init_vesting_block();
//         assert_noop!(
//             DoraRewards::initialize_contributors_list(Origin::root(), vec![(),()]),
//             Error::<Test>::NoContributorInList,
//         );
//     });
// }


/// input too many contributors number which is bigger than the `MaxContributorsNumber`
#[test]
fn initialize_too_many_contributors() {
    empty().execute_with(|| {
        // init lease block
        assert_noop!(
            // initialize the contributor list
            DoraRewards::initialize_contributors_list(
                Origin::root(),
                vec![
                    (1, 100u32.into()),
                    (2, 200u32.into()),
                    (3, 300u32.into()),
                    (4, 400u32.into()), 
                    (5, 500u32.into()), 
                    (6, 600u32.into()),
                ]
            ),
            Error::<Test>::TooManyContributors
        );
    });
}

/// initialize contributor list with sudo, but complete contributor with common account
/// this will dispatch a error with `BadOrigin`
#[test]
fn complete_contributor_with_common_user() {
    empty().execute_with(|| {
        // init lease block
        let init_block = DoraRewards::init_vesting_block();
        assert_ok!(DoraRewards::initialize_contributors_list(
            Origin::root(),
            vec![(1, 100u32.into()), (2, 200u32.into()), (3, 300u32.into())]
        ));

        assert_noop!(
            DoraRewards::complete_initialization(Origin::signed(0), init_block + VESTING),
            DispatchError::BadOrigin
        );
    });
}


/// set the invalid complete ending lease block(equal or below)
#[test]
fn set_invalid_ending_block(){
    empty().execute_with(|| {
        // init lease block: 2
		roll_to(2);
        let init_block = DoraRewards::init_vesting_block();
        assert_ok!(DoraRewards::initialize_contributors_list(
            Origin::root(),
            vec![(1, 100u32.into()), (2, 200u32.into()), (3, 300u32.into())]
        ));

        // ending block number equals the init block number
        assert_noop!(
            DoraRewards::complete_initialization(
                Origin::root(),
                init_block,
            ),
            Error::<Test>::InvalidEndingLeaseBlock,
        );

        // ending block number belows the init block number
        assert_noop!(
            DoraRewards::complete_initialization(
                Origin::root(),
                1,
            ),
            Error::<Test>::InvalidEndingLeaseBlock,
        );
    })
}

/// claim reward step by step
#[test]
fn claim_reward_step_by_step(){
    empty().execute_with(|| {
        // The init relay block gets inserted
		roll_to(2);
		let init_block = DoraRewards::init_vesting_block();

        assert_ok!(
            // initialize the contributor list
            DoraRewards::initialize_contributors_list(
                Origin::root(),
                vec![
                    (1, 330u32.into()), 
                    (2, 200u32.into()), 
                    (3, 323u32.into()),
                    (4, 400u32.into()), 
                ]
            )
        );

        // set the reward period (set 8)
        assert_ok!(DoraRewards::complete_initialization(
            Origin::root(),
            init_block + VESTING
        ));

        assert_eq!(DoraRewards::end_vesting_block(), 10);
        assert_eq!(DoraRewards::total_contributors(), 4);

        // test first claim reward (using account 4 as an example, 4 can get total reward is 1200)
        roll_to(4);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(4)));
        // 1200 * 20% + 960 * ((4-2) / 8) = 480
        assert_eq!(DoraRewards::rewards_info(&4).unwrap().claimed_reward, 480);

        // test some user not in the contributor list
        assert_noop!(
            DoraRewards::claim_rewards(Origin::signed(6)),
            Error::<Test>::NotInContributorList
        );

        // continue to claim reward by the block
        roll_to(5);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(4)));
        // 480 + 960 * ((5 - 4) / 8) = 600
        assert_eq!(DoraRewards::rewards_info(&4).unwrap().claimed_reward, 600);

        roll_to(6);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(4)));
        // 600 + 960 * ((6 - 5) / 8) = 720
        assert_eq!(DoraRewards::rewards_info(&4).unwrap().claimed_reward, 720);

        roll_to(7);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(4)));
        // 720 + 960 * ((7 - 6) / 8) = 840
        assert_eq!(DoraRewards::rewards_info(&4).unwrap().claimed_reward, 840);

        roll_to(8);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(4)));
        // 840 + 960 * ((8 - 7) / 8) = 960
        assert_eq!(DoraRewards::rewards_info(&4).unwrap().claimed_reward, 960);

        roll_to(9);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(4)));
        // 960 + 960 * ((8 - 7) / 8) = 1080
        assert_eq!(DoraRewards::rewards_info(&4).unwrap().claimed_reward, 1080);

        roll_to(12);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(4)));
        // 1080 + 960 * ((8 - 7) / 8) = 1200
        assert_eq!(DoraRewards::rewards_info(&4).unwrap().claimed_reward, 1200);

        // no rewards left
        roll_to(13);
		assert_noop!(
			DoraRewards::claim_rewards(Origin::signed(4)),
			Error::<Test>::NoLeftRewards
		);

        assert_ok!(DoraRewards::claim_rewards(Origin::signed(1)));
        assert_eq!(DoraRewards::rewards_info(&1).unwrap().claimed_reward, 990);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(2)));
        assert_eq!(DoraRewards::rewards_info(&2).unwrap().claimed_reward, 600);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(3)));
        assert_eq!(DoraRewards::rewards_info(&3).unwrap().claimed_reward, 969);

        assert_noop!(
			DoraRewards::claim_rewards(Origin::signed(1)),
			Error::<Test>::NoLeftRewards
		);
        assert_noop!(
			DoraRewards::claim_rewards(Origin::signed(2)),
			Error::<Test>::NoLeftRewards
		);
        assert_noop!(
			DoraRewards::claim_rewards(Origin::signed(3)),
			Error::<Test>::NoLeftRewards
		);
        
    })
}

#[test]
fn floating_point_arithmetic_works() {
	empty().execute_with(|| {
		// The init relay block gets inserted
		roll_to(2);
		let init_block = DoraRewards::init_vesting_block();

        assert_ok!(
            DoraRewards::initialize_contributors_list(
                Origin::root(),
                vec![
                    (4, 22u32.into()), 
                    (5, 1185u32.into()), 
                    (3, 25u32.into()),          // will receive 75 
                ]
            )
        );

		assert_ok!(DoraRewards::complete_initialization(
			Origin::root(),
			init_block + VESTING
		));
		assert_eq!(DoraRewards::total_contributors(), 3);

		assert_eq!(
			DoraRewards::rewards_info(&3).unwrap().total_reward,
			75u128
		);
        // claim the first reward : 20% 
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(3)));
		assert_eq!(
			DoraRewards::rewards_info(&3).unwrap().claimed_reward,
			15u128
		);

        // 60 * (1 / 8) = 7.5  each block
		// In this case there is no problem. Here we pay 7.5*2=15
		// Total claimed reward: 15+15 = 30
		roll_to(4);
		assert_ok!(DoraRewards::claim_rewards(Origin::signed(3)));
		assert_eq!(
			DoraRewards::rewards_info(&3).unwrap().claimed_reward,
			30u128
		);

		roll_to(5);
		// If we claim now we have to pay 7.5.    7 will be paid.
		assert_ok!(DoraRewards::claim_rewards(Origin::signed(3)));

		assert_eq!(
			DoraRewards::rewards_info(&3).unwrap().claimed_reward,
			37u128
		);
		roll_to(6);
		// Now we should pay 7.5. However the calculus will be:
		// Account 3 should have claimed 30 + 15 at this block, but
		// he only claimed 62. The payment is 8
		assert_ok!(DoraRewards::claim_rewards(Origin::signed(3)));
		assert_eq!(
			DoraRewards::rewards_info(&3).unwrap().claimed_reward,
			44u128
		);

        // pay 7.5 * 4 = 30, and have 1 left
        roll_to(10);
		assert_ok!(DoraRewards::claim_rewards(Origin::signed(3)));
		assert_eq!(
			DoraRewards::rewards_info(&3).unwrap().claimed_reward,
			74u128
		);

        // get the left reward, pay 1
        roll_to(11);
		assert_ok!(DoraRewards::claim_rewards(Origin::signed(3)));
		assert_eq!(
			DoraRewards::rewards_info(&3).unwrap().claimed_reward,
			75u128
		);

        roll_to(12);
        assert_noop!(
            DoraRewards::claim_rewards(Origin::signed(3)),
            Error::<Test>::NoLeftRewards
        );

        // test account 4
        assert_eq!(
			DoraRewards::rewards_info(&4).unwrap().claimed_reward,
			0u128
		);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(4)));
		assert_eq!(
			DoraRewards::rewards_info(&4).unwrap().claimed_reward,
			66u128
		);

        // test account 5
        assert_eq!(
			DoraRewards::rewards_info(&5).unwrap().claimed_reward,
			0u128
		);
        assert_ok!(DoraRewards::claim_rewards(Origin::signed(5)));
		assert_eq!(
			DoraRewards::rewards_info(&5).unwrap().claimed_reward,
			3555u128
		);

        assert_noop!(
            DoraRewards::claim_rewards(Origin::signed(5)),
            Error::<Test>::NoLeftRewards
        );
	});
}

// #[test]
// fn reward_below_vesting_period_works() {
// 	empty().execute_with(|| {
// 		// The init relay block gets inserted
// 		roll_to(2);
// 		let init_block = DoraRewards::init_vesting_block();
// 		assert_ok!(mock::Call::Utility(UtilityCall::batch_all {
// 			calls: vec![
// 				mock::Call::DoraRewards(crate::Call::initialize_reward_vec {
// 					rewards: vec![([4u8; 32].into(), Some(1), 1247)]
// 				}),
// 				mock::Call::DoraRewards(crate::Call::initialize_reward_vec {
// 					rewards: vec![([5u8; 32].into(), Some(2), 1247)]
// 				}),
// 				// We will work with this. This has 5/8=0.625 payable per block
// 				mock::Call::DoraRewards(crate::Call::initialize_reward_vec {
// 					rewards: vec![([3u8; 32].into(), Some(3), 6)]
// 				})
// 			]
// 		})
// 		.dispatch(Origin::root()));

// 		assert_ok!(DoraRewards::complete_initialization(
// 			Origin::root(),
// 			init_block + VESTING
// 		));

// 		assert_eq!(
// 			DoraRewards::accounts_payable(&3).unwrap().claimed_reward,
// 			1u128
// 		);

// 		// Block relay number is 2 post init initialization
// 		// Here we should pay floor(0.625*2)=1
// 		// Total claimed reward: 1+1 = 2
// 		roll_to(4);

// 		assert_ok!(DoraRewards::claim(Origin::signed(3)));

// 		assert_eq!(
// 			DoraRewards::accounts_payable(&3).unwrap().claimed_reward,
// 			2u128
// 		);
// 		roll_to(5);
// 		// If we claim now we have to pay floor(0.625) = 0
// 		assert_ok!(DoraRewards::claim(Origin::signed(3)));

// 		assert_eq!(
// 			DoraRewards::accounts_payable(&3).unwrap().claimed_reward,
// 			2u128
// 		);
// 		roll_to(6);
// 		// Now we should pay 1 again. The claimer should have claimed floor(0.625*4) + 1
// 		// but he only claimed 2
// 		assert_ok!(DoraRewards::claim(Origin::signed(3)));
// 		assert_eq!(
// 			DoraRewards::accounts_payable(&3).unwrap().claimed_reward,
// 			3u128
// 		);
// 		roll_to(10);
// 		// We pay the remaining
// 		assert_ok!(DoraRewards::claim(Origin::signed(3)));
// 		assert_eq!(
// 			DoraRewards::accounts_payable(&3).unwrap().claimed_reward,
// 			6u128
// 		);
// 		roll_to(11);
// 		// Nothing more to claim
// 		assert_noop!(
// 			DoraRewards::claim(Origin::signed(3)),
// 			Error::<Test>::RewardsAlreadyClaimed
// 		);
// 	});
// }