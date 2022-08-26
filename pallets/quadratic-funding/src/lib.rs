#![cfg_attr(not(feature = "std"), no_std)]

use codec::MaxEncodedLen;
use frame_support::{
    codec::{Decode, Encode},
    traits::{Currency, EnsureOrigin, Get, ReservableCurrency},
    BoundedVec, PalletId,
};
use orml_traits::{
    currency::TransferAll, MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency,
    MultiReservableCurrency, NamedMultiReservableCurrency,
};
pub use pallet::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
use scale_info::TypeInfo;
use sp_runtime::traits::{AccountIdConversion, Hash};
use sp_runtime::RuntimeDebug;
use sp_std::{convert::TryInto, vec, vec::Vec};

mod benchmarking;

pub mod weights;

pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct Project<AccountId, BoundedString> {
    pub total_votes: u128,
    pub grants: u128,
    pub support_area: u128,
    pub support_fund: u128,
    pub name: BoundedString,
    pub owner: AccountId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct Round<AccountId, BoundedString, CurrencyIdOf> {
    pub name: BoundedString,
    pub currency_id: CurrencyIdOf,
    pub ongoing: bool,
    pub support_pool: u128,
    pub pre_tax_support_pool: u128,
    pub total_support_area: u128,
    pub total_tax: u128,
    pub round_reserve: u128,
    pub admin: AccountId,
}

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type DoraBalance<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
pub(crate) type CurrencyIdOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<
    <T as frame_system::Config>::AccountId,
>>::CurrencyId;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Currency to transfer assets
        type MultiCurrency: TransferAll<Self::AccountId>
            + MultiCurrencyExtended<Self::AccountId>
            + MultiLockableCurrency<Self::AccountId>
            + MultiReservableCurrency<Self::AccountId>
            + NamedMultiReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Origin from which admin must come.
        type AdminOrigin: EnsureOrigin<Self::Origin>;

        // What to do with slashed funds.
        // type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;
        // type Slashed: AccountId;

        /// UnitOfVote, 0.001 Unit token
        type UnitOfVote: Get<u128>;

        /// Number of base unit for each vote
        type NumberOfUnitPerVote: Get<u128>;

        /// The ration of fee based on the number of unit
        type FeeRatioPerVote: Get<u128>;

        /// The minimum length of name [project_name, round_name]
        type NameMinLength: Get<u32>;

        /// The maximum length of name [project_name, round_name]
        type NameMaxLength: Get<u32>;

        type ReserveUnit: Get<u128>;

        /// Infomation on runtime weights.
        type WeightInfo: WeightInfo;
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
    pub(super) type Rounds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        Round<
            <T as frame_system::Config>::AccountId,
            BoundedVec<u8, T::NameMaxLength>,
            CurrencyIdOf<T>,
        >,
    >;

    #[pallet::storage]
    #[pallet::getter(fn projects)]
    pub(super) type Projects<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32,
        Blake2_128Concat,
        T::Hash,
        Project<<T as frame_system::Config>::AccountId, BoundedVec<u8, T::NameMaxLength>>,
    >;

    #[pallet::storage]
    pub(super) type ProjectVotes<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::Hash, Blake2_128Concat, T::AccountId, u128>;

    #[pallet::storage]
    #[pallet::getter(fn round_participants)]
    pub(super) type RoundParticipants<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, bool>;

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
        DonateSucceed(u32, T::AccountId, BalanceOf<T>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        BadMetadata,
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
        RoundNameTooLong,
        RoundNameTooShort,
        DuplicateRound,
        MismatchingCurencyId,
        InsufficientReserveDora,
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
        #[pallet::weight(T::WeightInfo::donate())]
        pub fn donate(
            origin: OriginFor<T>,
            round_id: u32,
            #[pallet::compact] amount: BalanceOf<T>,
            currency_id: CurrencyIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;
            ensure!(
                Rounds::<T>::contains_key(&round_id),
                Error::<T>::RoundNotExist
            );
            let round = Rounds::<T>::get(round_id).unwrap();
            ensure!(true == round.ongoing, Error::<T>::RoundHasEnded);
            ensure!(
                currency_id == round.currency_id,
                Error::<T>::MismatchingCurencyId
            );
            // the minimum unit, make sure the donate is greater than this
            let min_unit_number = Self::cal_amount(1u128, false);
            let amount_number = Self::balance_to_u128(amount);
            let fee_number = T::FeeRatioPerVote::get()
                .checked_mul(amount_number / T::NumberOfUnitPerVote::get())
                .unwrap();
            ensure!(
                amount_number > min_unit_number,
                Error::<T>::DonationTooSmall
            );

            let _ = T::MultiCurrency::transfer(
                currency_id,
                &who,
                &Self::account_id(),
                Self::u128_to_balance(fee_number),
            )?;
            let _ = T::MultiCurrency::transfer(
                currency_id,
                &who,
                &Self::round_admin_account(round_id),
                Self::u128_to_balance(amount_number - fee_number),
            )?;
            // add deposit to pallet account.
            // update the round
            Rounds::<T>::mutate(round_id, |rnd| match rnd {
                Some(round) => {
                    let ptsp = round.pre_tax_support_pool;
                    let sp = round.support_pool;
                    let tt = round.total_tax;
                    round.pre_tax_support_pool = amount_number.checked_add(ptsp).unwrap();
                    round.support_pool = (amount_number - fee_number).checked_add(sp).unwrap();
                    round.total_tax = fee_number.checked_add(tt).unwrap();
                }
                _ => (),
            });
            Self::deposit_event(Event::DonateSucceed(round_id, who, amount));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::start_round())]
        pub fn start_round(
            origin: OriginFor<T>,
            round_id: u32,
            currency_id: CurrencyIdOf<T>,
            name: Vec<u8>,
            admin: T::AccountId,
            round_reserve: u128,
        ) -> DispatchResultWithPostInfo {
            // Only amdin can control the round
            T::AdminOrigin::ensure_origin(origin)?;
            ensure!(
                name.len() >= T::NameMinLength::get().try_into().unwrap(),
                Error::<T>::RoundNameTooShort
            );
            ensure!(
                name.len() <= T::NameMaxLength::get().try_into().unwrap(),
                Error::<T>::RoundNameTooLong
            );
            ensure!(
                !Rounds::<T>::contains_key(&round_id),
                Error::<T>::RoundExisted
            );

            let bounded_name: BoundedVec<u8, T::NameMaxLength> = name
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::BadMetadata)?;
            let round = Round {
                ongoing: true,
                name: bounded_name,
                currency_id: currency_id,
                support_pool: 0,
                pre_tax_support_pool: 0,
                total_support_area: 0,
                total_tax: 0,
                round_reserve: round_reserve,
                admin: admin.clone(),
            };
            Rounds::<T>::insert(round_id, round);
            Self::deposit_event(Event::RoundStarted(round_id));
            Ok(().into())
        }

        // TODO add funding grants, Reserve number/by round
        /// End an `ongoing` round and distribute the funds in sponsor pool, any invalid index or round status will cause errors
        #[pallet::weight(T::WeightInfo::end_round())]
        pub fn end_round(origin: OriginFor<T>, round_id: u32) -> DispatchResultWithPostInfo {
            // Only amdin can control the round
            T::AdminOrigin::ensure_origin(origin)?;
            ensure!(
                Rounds::<T>::contains_key(&round_id),
                Error::<T>::RoundNotExist
            );
            let mut round = Rounds::<T>::get(round_id).unwrap();
            ensure!(true == round.ongoing, Error::<T>::RoundHasEnded);
            let area = round.total_support_area;
            let pool = round.support_pool;
            // update the support fund
            for (hash, _) in Projects::<T>::iter_prefix(round_id) {
                if area > 0 {
                    Projects::<T>::mutate(round_id, hash, |poj| match poj {
                        Some(project) => {
                            project.support_fund =
                                project.support_area.checked_mul(pool / area).unwrap();
                        }
                        _ => (),
                    });
                }
            }
            // unreserve the DORA to voters and update states
            for (voter, _) in RoundParticipants::<T>::iter_prefix(round_id) {
                let reserve_num = T::ReserveUnit::get()
                    .checked_mul(round.round_reserve)
                    .unwrap();
                let reserve_balance = TryInto::<DoraBalance<T>>::try_into(reserve_num)
                    .ok()
                    .unwrap();
                // ReservableCurrency::unreserve does not fail (it will lock up as much as amount)
                T::Currency::unreserve(&voter, reserve_balance);
            }
            // Maybe this is unnecessary as round ended, voter can not do malicious attack
            let _ = RoundParticipants::<T>::clear_prefix(round_id, u32::MAX, None);

            // update round status
            round.ongoing = false;
            Rounds::<T>::insert(round_id, round);
            Self::deposit_event(Event::RoundEnded(round_id));
            Ok(().into())
        }

        /// Register a project in an ongoing round, so that it can be voted
        #[pallet::weight(T::WeightInfo::register_project())]
        pub fn register_project(
            origin: OriginFor<T>,
            round_id: u32,
            hash: T::Hash,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                name.len() >= T::NameMinLength::get().try_into().unwrap(),
                Error::<T>::ProjectNameTooShort
            );
            ensure!(
                name.len() <= T::NameMaxLength::get().try_into().unwrap(),
                Error::<T>::ProjectNameTooLong
            );
            ensure!(
                Rounds::<T>::contains_key(&round_id),
                Error::<T>::RoundNotExist
            );
            let round = Rounds::<T>::get(round_id).unwrap();
            ensure!(true == round.ongoing, Error::<T>::RoundHasEnded);
            ensure!(
                !Projects::<T>::contains_key(&round_id, &hash),
                Error::<T>::DuplicateProject
            );

            let bounded_name: BoundedVec<u8, T::NameMaxLength> = name
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::BadMetadata)?;
            let project = Project {
                total_votes: 0,
                grants: 0,
                support_area: 0,
                support_fund: 0,
                name: bounded_name,
                owner: who.clone(),
            };
            Projects::<T>::insert(round_id, hash, project);
            Self::deposit_event(Event::ProjectRegistered(hash, who));
            Ok(().into())
        }

        /// Vote to a project, this function will transfer corresponding amount of token per your input ballot
        #[pallet::weight(T::WeightInfo::vote())]
        pub fn vote(
            origin: OriginFor<T>,
            currency_id: CurrencyIdOf<T>,
            round_id: u32,
            hash: T::Hash,
            ballot: u128,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // check whether this round still ongoing
            ensure!(
                Rounds::<T>::contains_key(&round_id),
                Error::<T>::RoundNotExist
            );
            let round = Rounds::<T>::get(round_id).unwrap();
            ensure!(true == round.ongoing, Error::<T>::RoundHasEnded);
            ensure!(
                Projects::<T>::contains_key(&round_id, &hash),
                Error::<T>::ProjectNotExist
            );
            ensure!(
                currency_id == round.currency_id,
                Error::<T>::MismatchingCurencyId
            );
            ensure!(ballot > 0, Error::<T>::InvalidBallot);

            // need to calculate hash of project hash and round_id combination here to avoid conflicts of projects in different rounds
            let vote_hash = T::Hashing::hash_of(&(&hash, &round_id));
            let voted = match ProjectVotes::<T>::get(vote_hash, &who) {
                Some(val) => val,
                None => 0,
            };

            // check whether staked
            let staked = RoundParticipants::<T>::get(round_id, &who);
            if round.round_reserve > 0 && staked == None {
                let reserve_num = T::ReserveUnit::get()
                    .checked_mul(round.round_reserve)
                    .unwrap();
                let reserve_balance = TryInto::<DoraBalance<T>>::try_into(reserve_num)
                    .ok()
                    .unwrap();
                T::Currency::reserve(&who, reserve_balance)
                    .map_err(|_| Error::<T>::InsufficientReserveDora)?;
                RoundParticipants::<T>::insert(round_id, &who, true);
            }
            let cost = Self::cal_cost(voted.clone(), ballot);
            let amount = Self::cal_amount(cost, false);
            let fee = Self::cal_amount(cost, true);

            let _ = T::MultiCurrency::transfer(
                currency_id,
                &who,
                &Self::account_id(),
                Self::u128_to_balance(fee),
            )?;
            // transfer first, update last, as transfer will ensure the free balance is enough
            let _ = T::MultiCurrency::transfer(
                currency_id,
                &who,
                &Self::round_admin_account(round_id),
                Self::u128_to_balance(amount - fee),
            )?;
            // update the project and corresponding round
            ProjectVotes::<T>::insert(vote_hash, &who, ballot + voted);
            Projects::<T>::mutate(round_id, hash, |poj| {
                match poj {
                    Some(project) => {
                        let support_area = ballot.checked_mul(project.total_votes - voted).unwrap();
                        project.support_area =
                            support_area.checked_add(project.support_area).unwrap();
                        project.total_votes += ballot;
                        project.grants += amount - fee;
                        //debug::info!("Total votes: {:?}, Current votes: {:?}, Support Area: {:?},Est cost: {:?}",
                        // poj.total_votes, voted, support_area, cost);
                        Rounds::<T>::mutate(round_id, |rnd| match rnd {
                            Some(round) => {
                                let tsa = round.total_support_area;
                                let tt = round.total_tax;
                                round.total_support_area = support_area.checked_add(tsa).unwrap();
                                round.total_tax = fee.checked_add(tt).unwrap();
                            }
                            _ => (),
                        });
                    }
                    _ => (),
                }
            });
            Self::deposit_event(Event::VoteSucceed(hash, who, ballot));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    // Add public immutables and private mutables.

    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    /// get corresponding accounts
    pub fn round_admin_account(round_id: u32) -> T::AccountId {
        let round = Rounds::<T>::get(round_id).unwrap();
        round.admin
    }

    pub fn cal_cost(voted: u128, ballot: u128) -> u128 {
        let mut points = ballot.checked_mul(ballot.checked_add(1).unwrap()).unwrap() / 2;
        points = points
            .checked_add(ballot.checked_mul(voted).unwrap())
            .unwrap();
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
        TryInto::<BalanceOf<T>>::try_into(cost).ok().unwrap()
    }

    pub fn balance_to_u128(balance: BalanceOf<T>) -> u128 {
        TryInto::<u128>::try_into(balance).ok().unwrap()
    }

    // TODO: There is a bug for serde_json, can not use u128 https://github.com/paritytech/substrate/issues/4641
    pub fn vote_cost(who: T::AccountId, round_id: u32, hash: T::Hash, ballot: u32) -> u32 {
        // need to calculate hash of project hash and round_id combination here to avoid conflicts of projects in different rounds
        let vote_hash = T::Hashing::hash_of(&(&hash, &round_id));
        let voted = ProjectVotes::<T>::get(vote_hash, &who).unwrap();
        TryInto::<u32>::try_into(Self::cal_cost(voted, ballot.into()))
            .ok()
            .unwrap()
    }

    // TODO, using struct is a little complicate, use tuple instead
    // (project_id, total_votes, grants, support_grants)
    pub fn projects_per_round(round_id: u32) -> Vec<(T::Hash, u32, u32, u32)> {
        let mut projects = vec![];
        let round = Rounds::<T>::get(round_id).unwrap();
        let area = round.total_support_area;
        let pool = round.support_pool;
        for (hash, project) in Projects::<T>::iter_prefix(round_id) {
            let mut sg = 0;
            if area > 0 {
                sg = project.support_area.checked_mul(pool / area).unwrap()
            }
            let total_votes = TryInto::<u32>::try_into(project.total_votes).ok().unwrap();
            let grants =
                TryInto::<u32>::try_into(project.grants.checked_div(T::UnitOfVote::get()).unwrap())
                    .ok()
                    .unwrap();
            let support_grants =
                TryInto::<u32>::try_into(sg.checked_div(T::UnitOfVote::get()).unwrap())
                    .ok()
                    .unwrap();
            projects.push((hash, total_votes, grants, support_grants))
        }
        projects
    }
}
