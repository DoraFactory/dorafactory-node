#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
use frame_support::{
    traits::{Currency, Get, ExistenceRequirement::{KeepAlive}},
    codec::{Encode, Decode}
};
use sp_std::{vec, vec::Vec, convert::{TryInto}};
use sp_runtime::traits::{Hash};
use core_services::{DoraUserOrigin, DoraPay};
use scale_info::TypeInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct Project<AccountId> {
    pub total_votes: u128,
    pub grants: u128,
    pub support_area: u128,
    pub withdrew: u128,
    pub name: Vec<u8>,
    pub owner: AccountId,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct Round {
    pub name: Vec<u8>,
    pub ongoing: bool,
    pub support_pool: u128,
    pub pre_tax_support_pool: u128,
    pub total_support_area: u128,
    pub total_tax: u128,
}

type ProjectOf<T> = Project<<T as frame_system::Config>::AccountId>;
type BalanceOf<T> = <<T as pallet_dao_core::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_dao_core::Config{
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Origin from which admin must come.
        type AdminOrigin: EnsureOrigin<Self::Origin>;

        /// Dora services
        type DoraUserOrigin: DoraUserOrigin<AccountId = Self::AccountId, OrgId = u32, AppId = u8>;
        type DoraPay: DoraPay<AccountId = Self::AccountId, Balance= BalanceOf<Self>, AppId = u8>;
        type AppId: Get<u8>;

        /// UnitOfVote, 0.001 Unit token
        type UnitOfVote: Get<u128>;

        /// Number of base unit for each vote
        type NumberOfUnitPerVote: Get<u128>;

        /// The ration of fee based on the number of unit
        type FeeRatioPerVote: Get<u128>;

        /// The minimum length of project name
        type NameMinLength: Get<usize>;

        /// The maximum length of project name
        type NameMaxLength: Get<usize>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn rounds)]
    // Learn more about declaring storage items:
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
    pub(super) type Rounds<T> = StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, u32, Round, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn projects)]
    pub(super) type Projects<T: Config> = StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::Hash, ProjectOf<T>, ValueQuery>;

    #[pallet::storage]
    pub(super) type ProjectVotes<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::Hash, Blake2_128Concat, T::AccountId, u128, ValueQuery>;


    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [project_hash, who]
        ProjectRegistered(T::Hash, T::AccountId),
        /// parameters. [project_hash, balance of cost]
        VoteCost(T::Hash, u128),
        /// parameters. [project_hash, who, number of ballots]
        VoteSucceed(T::Hash, T::AccountId, u128),
        /// parameters. [round_id]
        RoundStarted(u32),
        /// parameters. [round_id]
        RoundEnded(u32),
        /// parameters. [round_id, who, amount]
        DonateSucceed(u32, T::AccountId, u128),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        DuplicateProject,
        ProjectNotExist,
        ProjectNameTooLong,
        ProjectNameTooShort,
        InvalidBallot,
        DonationTooSmall,
        RoundExisted,
        RoundNotExist,
        RoundHasEnded,
        DuplicateRound,
        NotValidOrgMember,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn donate(origin: OriginFor<T>, round_id: u32, org_id: u32, #[pallet::compact] amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;
            // anyone can donate
            // let _ = T::DoraUserOrigin::ensure_valid(who.clone(), org_id, T::AppId::get())?;

            ensure!(Rounds::<T>::contains_key(&org_id, &round_id), Error::<T>::RoundNotExist);
            let round = Rounds::<T>::get(org_id, round_id);
            ensure!(true == round.ongoing, Error::<T>::RoundHasEnded);
            // the minimum unit, make sure the donate is greater than this
            let min_unit_number = Self::cal_amount(1u128, false);
            let amount_number = Self::balance_to_u128(amount);
            let fee_number = T::FeeRatioPerVote::get().checked_mul(amount_number / T::NumberOfUnitPerVote::get()).unwrap();
            ensure!(amount_number > min_unit_number, Error::<T>::DonationTooSmall);
            let _ = T::DoraPay::charge(who.clone(), amount, T::AppId::get());
            // update the round
            Rounds::<T>::mutate(org_id, round_id, |rnd| {
                let ptsp = rnd.pre_tax_support_pool;
                let sp = rnd.support_pool;
                let tt = rnd.total_tax;
                rnd.pre_tax_support_pool = amount_number.checked_add(ptsp).unwrap();
                rnd.support_pool = (amount_number-fee_number).checked_add(sp).unwrap();
                rnd.total_tax = fee_number.checked_add(tt).unwrap();
            });
            Self::deposit_event(Event::DonateSucceed(round_id, who, Self::balance_to_u128(amount)));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn start_round(origin: OriginFor<T>, round_id: u32, org_id: u32, name: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let _ = T::DoraUserOrigin::ensure_valid(who.clone(), org_id, T::AppId::get())?;

            ensure!(<pallet_dao_core::Pallet<T>>::validate_member(who.clone(), org_id), Error::<T>::NotValidOrgMember);
            ensure!(!Rounds::<T>::contains_key(&org_id, &round_id), Error::<T>::RoundExisted);
            let round = Round {
                ongoing: true,
                name: name,
                support_pool: 0,
                pre_tax_support_pool: 0,
                total_support_area: 0,
                total_tax: 0
            };
            Rounds::<T>::insert(org_id, round_id, round);
            Self::deposit_event(Event::RoundStarted(round_id));
            Ok(().into())
        }

        /// End an `ongoing` round and distribute the funds in sponsor pool, any invalid index or round status will cause errors
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn end_round(origin: OriginFor<T>, round_id: u32, org_id: u32) -> DispatchResultWithPostInfo {
            // Only amdin can control the round 
            // T::AdminOrigin::ensure_origin(origin)?;
            let who = ensure_signed(origin)?;
            let _ = T::DoraUserOrigin::ensure_valid(who.clone(), org_id, T::AppId::get())?;

            ensure!(<pallet_dao_core::Pallet<T>>::validate_member(who.clone(), org_id), Error::<T>::NotValidOrgMember);
            ensure!(Rounds::<T>::contains_key(&org_id, &round_id), Error::<T>::RoundNotExist);
            let mut round = Rounds::<T>::get(org_id, round_id);
            let project_key = u64::MIN.checked_add(round_id.into()).unwrap().checked_add(org_id.into()).unwrap();
            ensure!(true == round.ongoing, Error::<T>::RoundHasEnded);
            let area = round.total_support_area;
            let pool = round.support_pool;
            for (_hash, mut project) in Projects::<T>::iter_prefix(project_key) {
                if area > 0 {
                    let total = project.grants;
                    project.grants = total.checked_add(
                        project.support_area.checked_mul(pool/area).unwrap()
                    ).unwrap();
                }
                //debug::info!("Hash: {:?}, Total votes: {:?}, Grants: {:?}", hash, project.total_votes, project.grants);
                // reckon the final grants
                let _ = T::DoraPay::withdraw(
                    project.owner,
                    Self::u128_to_balance(project.grants),
                    T::AppId::get()
                );
            }
            round.ongoing = false;
            Rounds::<T>::insert(org_id, round_id, round);
            Self::deposit_event(Event::RoundEnded(round_id));
            Ok(().into())
        }

        /// Register a project in an ongoing round, so that it can be voted
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn register_project(origin: OriginFor<T>, round_id: u32, org_id: u32, hash: T::Hash, name: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let _ = T::DoraUserOrigin::ensure_valid(who.clone(), org_id, T::AppId::get())?;

            let project_hash = T::Hashing::hash_of(&(&hash, &org_id));
            let project_key = u64::MIN.checked_add(round_id.into()).unwrap().checked_add(org_id.into()).unwrap();
            ensure!(<pallet_dao_core::Pallet<T>>::validate_member(who.clone(), org_id), Error::<T>::NotValidOrgMember);
            ensure!(name.len() >= T::NameMinLength::get(), Error::<T>::ProjectNameTooShort);
            ensure!(name.len() <= T::NameMaxLength::get(), Error::<T>::ProjectNameTooLong);
            ensure!(!Projects::<T>::contains_key(&project_key, &project_hash), Error::<T>::DuplicateProject);
            let project = Project {
                total_votes: 0,
                grants: 0,
                support_area: 0,
                withdrew: 0,
                name: name,
                owner: who.clone(),
            };
            Projects::<T>::insert(project_key, project_hash, project);
            Self::deposit_event(Event::ProjectRegistered(hash, who));
            Ok(().into())
        }

        /// Vote to a project, this function will transfer corresponding amount of token per your input ballot
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn vote(origin: OriginFor<T>, round_id: u32, org_id: u32, hash: T::Hash, ballot: u128) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // OrgUser can cross vote, no need to verify here
            // let _ = T::DoraUserOrigin::ensure_valid(who.clone(), org_id, T::AppId::get())?;

            let project_hash = T::Hashing::hash_of(&(&hash, &org_id));
            let project_key = u64::MIN.checked_add(round_id.into()).unwrap().checked_add(org_id.into()).unwrap();
            ensure!(Projects::<T>::contains_key(&project_key, &project_hash), Error::<T>::ProjectNotExist);
            ensure!(ballot > 0, Error::<T>::InvalidBallot);
            // check whether this round still ongoing
            ensure!(Rounds::<T>::contains_key(&org_id, &round_id), Error::<T>::RoundNotExist);
            let round = Rounds::<T>::get(org_id, round_id);
            ensure!(true == round.ongoing, Error::<T>::RoundHasEnded);

            // need to calculate hash of project hash and round_id combination here to avoid conflicts of projects in different rounds
            let vote_hash = T::Hashing::hash_of(&(&hash, &round_id));
            let voted = ProjectVotes::<T>::get(vote_hash, &who);
            let cost = Self::cal_cost(voted, ballot);
            let amount = Self::cal_amount(cost, false);
            let fee = Self::cal_amount(cost, true);
            // transfer first, update last, as transfer will ensure the free balance is enough
            let _ = T::DoraPay::charge(who.clone(), Self::u128_to_balance(amount), T::AppId::get());

            // update the project and corresponding round
            ProjectVotes::<T>::insert(vote_hash, &who, ballot+voted);
            Projects::<T>::mutate(project_key, project_hash, |poj| {
                let support_area = ballot.checked_mul(poj.total_votes - voted).unwrap();
                poj.support_area = support_area.checked_add(poj.support_area).unwrap();
                poj.total_votes += ballot;
                poj.grants += amount - fee;
                //debug::info!("Total votes: {:?}, Current votes: {:?}, Support Area: {:?},Est cost: {:?}",
                // poj.total_votes, voted, support_area, cost);
                Rounds::<T>::mutate(org_id, round_id, |rnd| {
                    let tsa = rnd.total_support_area;
                    let tt = rnd.total_tax;
                    rnd.total_support_area = support_area.checked_add(tsa).unwrap();
                    rnd.total_tax = fee.checked_add(tt).unwrap();
                });
            });
            Self::deposit_event(Event::VoteSucceed(hash, who, ballot));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    // Add public immutables and private mutables.

    /// refer https://github.com/paritytech/substrate/blob/743accbe3256de2fc615adcaa3ab03ebdbbb4dbd/frame/treasury/src/lib.rs#L351
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.

    pub fn cal_cost(voted: u128, ballot: u128) -> u128 {
        let mut points = ballot.checked_mul(ballot.checked_add(1).unwrap()).unwrap() / 2; 
        points = points.checked_add(ballot.checked_mul(voted).unwrap()).unwrap();
        return points;
    }

    pub fn cal_amount(amount: u128, is_fee: bool) -> u128 {
        let uov = T::UnitOfVote::get();
        let nup = T::NumberOfUnitPerVote::get();
        let frpv = T::FeeRatioPerVote::get();
        if is_fee { 
            uov.checked_mul(frpv).unwrap().checked_mul(amount).unwrap() 
        } else {
            uov.checked_mul(nup).unwrap().checked_mul(amount).unwrap()
        }
    }

    pub fn u128_to_balance(cost: u128) -> BalanceOf<T> {
        TryInto::<BalanceOf::<T>>::try_into(cost).ok().unwrap()
    }

    pub fn balance_to_u128(balance: BalanceOf<T>) -> u128 {
        TryInto::<u128>::try_into(balance).ok().unwrap()
    }

    // TODO: There is a bug for serde_json, can not use u128 https://github.com/paritytech/substrate/issues/4641
    pub fn vote_cost(who: T::AccountId, round_id:u32, hash: T::Hash, ballot: u32) -> u32 {
        // need to calculate hash of project hash and round_id combination here to avoid conflicts of projects in different rounds
        let vote_hash = T::Hashing::hash_of(&(&hash, &round_id));
        let voted = ProjectVotes::<T>::get(vote_hash, &who);
        TryInto::<u32>::try_into(Self::cal_cost(voted, ballot.into())).ok().unwrap()
    }

    // TODO, using struct is a little complicate, use tuple instead
    // (project_id, total_votes, grants, support_grants)
    pub fn projects_per_round(round_id:u32, org_id: u32) -> Vec<(T::Hash, u32, u32, u32)> {
        let mut projects  = vec![];
        let round = Rounds::<T>::get(org_id, round_id);
        let area = round.total_support_area;
        let pool = round.support_pool;
        let project_key = u64::MIN.checked_add(round_id.into()).unwrap().checked_add(org_id.into()).unwrap();
        for (hash, project) in Projects::<T>::iter_prefix(project_key) {
            let mut sg = 0;
            if area > 0 {
                sg = project.support_area.checked_mul(pool/area).unwrap()
            }
            let total_votes = TryInto::<u32>::try_into(project.total_votes).ok().unwrap();
            let grants = TryInto::<u32>::try_into(project.grants.checked_div(T::UnitOfVote::get()).unwrap()).ok().unwrap();
            let support_grants = TryInto::<u32>::try_into(sg.checked_div(T::UnitOfVote::get()).unwrap()).ok().unwrap();
            projects.push((hash, total_votes, grants, support_grants))
        }
        projects
    }
}
