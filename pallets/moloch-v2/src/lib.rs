#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    codec::{Decode, Encode, MaxEncodedLen},
    traits::{
        Currency,
        ExistenceRequirement::{AllowDeath, KeepAlive},
        Get, OnUnbalanced, ReservableCurrency,
    },
    PalletId,
};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug};
use sp_std::{convert::TryInto, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// TODO: Not support enum in storage
#[derive(Encode, Decode, Clone, PartialEq)]
pub enum Vote {
    // default value, counted as abstention
    Null,
    Yes,
    No,
}
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct Member<AccountId> {
    // the # of shares assigned to this member
    pub shares: u128,
    // the loot amount available to this member (combined with shares on ragequit)
    pub loot: u128,
    // highest proposal index # on which the member voted YES
    pub highest_index_yes_vote: u128,
    // always true once a member has been created
    pub exists: bool,
    // the key responsible for submitting proposals and voting - defaults to member address unless updated
    pub delegate_key: AccountId,
    // set to proposalIndex of a passing guild kick proposal for this member, prevents voting on and sponsoring proposals
    pub jailed_at: u128,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo, RuntimeDebug)]
pub struct Proposal<AccountId> {
    // the account that submitted the proposal (can be non-member)
    pub proposer: AccountId,
    // the applicant who wishes to become a member - this key will be used for withdrawals (doubles as guild kick target for gkick proposals)
    pub applicant: AccountId,
    // the member that sponsored the proposal (moving it into the queue)
    pub sponsor: AccountId,
    // the # of shares the applicant is requesting
    pub shares_requested: u128,
    // the # of loot the applicant is requesting
    pub loot_requested: u128,
    // amount of tokens requested as payment
    pub payment_requested: u128,
    // amount of tokens offered as tribute
    pub tribute_offered: u128,
    // [sponsored, processed, didPass, cancelled, whitelist, guildkick]
    pub flags: [bool; 6],
    // the period in which voting can start for this proposal
    pub starting_period: u128,
    // the total number of YES votes for this proposal
    pub yes_votes: u128,
    // the total number of NO votes for this proposal
    pub no_votes: u128,
    // proposal details - Must be ascii chars, limited length
    pub details: Vec<u8>,
    // the maximum # of total shares encountered at a yes vote on this proposal
    pub max_total_shares_at_yes: u128,
}

type MemberOf<T> = Member<<T as frame_system::Config>::AccountId>;
type ProposalOf<T> = Proposal<<T as frame_system::Config>::AccountId>;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: pallet_timestamp::Config + frame_system::Config {
        // The runtime must supply this pallet with an Event type that satisfies the pallet's requirements.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Origin from which admin must come.
        type AdminOrigin: EnsureOrigin<Self::Origin>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// What to do with slashed funds.
        type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

        // maximum length of voting period
        type MaxVotingPeriodLength: Get<u128>;

        // maximum length of grace period
        type MaxGracePeriodLength: Get<u128>;

        // maximum dilution bound
        type MaxDilutionBound: Get<u128>;

        // maximum number of shares
        type MaxShares: Get<u128>;
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        VotingPeriodLengthTooBig,
        DilutionBoundTooBig,
        GracePeriodLengthTooBig,
        NoEnoughProposalDeposit,
        NoEnoughShares,
        NoEnoughLoot,
        NotMember,
        NotStandardProposal,
        NotKickProposal,
        NotProposalProposer,
        SharesOverFlow,
        ProposalNotExist,
        ProposalNotStart,
        ProposalNotReady,
        ProposalHasSponsored,
        ProposalHasProcessed,
        ProposalHasAborted,
        ProposalNotProcessed,
        PreviousProposalNotProcessed,
        ProposalExpired,
        InvalidVote,
        MemberHasVoted,
        NoOverwriteDelegate,
        NoOverwriteMember,
        NoCustodyFound,
        MemberInJail,
        MemberNotInJail,
    }

    // Events.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [proposalIndex, delegateKey, memberAddress, applicant, tokenTribute, sharesRequested]
        SubmitProposal(u128, T::AccountId, T::AccountId, T::AccountId, u128, u128),
        /// parameters. [proposalIndex, delegateKey, memberAddress, uintVote]
        SubmitVote(u128, T::AccountId, T::AccountId, u8),
        /// parameters. [proposalIndex, applicant, memberAddress, tokenTribute, sharesRequested, didPass]
        ProcessProposal(u128, T::AccountId, T::AccountId, u128, u128, bool),
        /// parameters. [memberAddress, sharesToBurn]
        Ragequit(T::AccountId, u128),
        /// parameters. [proposalIndex, applicantAddress]
        Abort(u128, T::AccountId),
        /// parameters. [memberAddress, newDelegateKey]
        UpdateDelegateKey(T::AccountId, T::AccountId),
        /// parameters. [summoner, shares]
        SummonComplete(T::AccountId, u128),
        /// parameters. [totalShares, dilutionBond, maxTotalSharesVoteAtYes]
        DilutionBoundExeceeds(u128, u128, u128),
        // parameters. [currentReserved, requiredReserved]
        //CustodyBalanceOutage(Balance, Balance),
        // CustodySucceeded(AccountId, Balance),
    }

    // Learn more about declaring storage items:
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
    #[pallet::storage]
    #[pallet::getter(fn totoal_shares)]
    pub(super) type TotalShares<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn totoal_loot)]
    pub(super) type TotalLoot<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn period_duration)]
    pub(super) type PeriodDuration<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn voting_period_length)]
    pub(super) type VotingPeriodLength<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn grace_period_length)]
    pub(super) type GracePeriodLength<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_count)]
    pub(super) type ProposalCount<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_deposit)]
    pub(super) type ProposalDeposit<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn dilution_bound)]
    pub(super) type DilutionBound<T> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn processing_reward)]
    pub(super) type ProcessingReward<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn summon_time)]
    pub(super) type SummonTime<T: Config> = StorageValue<_, T::Moment, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub(super) type Members<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, MemberOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn address_of_delegate)]
    pub(super) type AddressOfDelegates<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_queue)]
    pub(super) type ProposalQueue<T> = StorageValue<_, Vec<u128>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_vote)]
    pub(super) type ProposalVotes<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u128, Blake2_128Concat, T::AccountId, u8, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub(super) type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, u128, ProposalOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposed_to_kick)]
    pub(super) type ProsedToKick<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    // #[pallet::genesis_build]
    // impl<T: Config> GenesisBuild<T> for GenesisConfig {
    // 	fn build(&self) {
    // 		// Create pallet's internal account
    // 		let _ = T::Currency::make_free_balance_be(
    // 			&<Module<T>>::account_id(),
    // 			T::Currency::minimum_balance(),
    // 		);
    // 		let _ = T::Currency::make_free_balance_be(
    // 			&<Module<T>>::custody_account(),
    // 			T::Currency::minimum_balance(),
    // 		);
    // 	}
    // }
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.

        /// Summon a group or orgnization
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn summon(
            origin: OriginFor<T>,
            period_duration: u32,
            voting_period_length: u128,
            grace_period_length: u128,
            dilution_bound: u128,
            #[pallet::compact] proposal_deposit: BalanceOf<T>,
            #[pallet::compact] processing_reward: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                voting_period_length <= T::MaxVotingPeriodLength::get(),
                Error::<T>::VotingPeriodLengthTooBig
            );
            ensure!(
                grace_period_length <= T::MaxGracePeriodLength::get(),
                Error::<T>::GracePeriodLengthTooBig
            );
            ensure!(
                dilution_bound <= T::MaxDilutionBound::get(),
                Error::<T>::DilutionBoundTooBig
            );
            ensure!(
                proposal_deposit >= processing_reward,
                Error::<T>::NoEnoughProposalDeposit
            );

            SummonTime::<T>::put(pallet_timestamp::Pallet::<T>::now());
            PeriodDuration::<T>::put(period_duration);
            VotingPeriodLength::<T>::put(voting_period_length);
            GracePeriodLength::<T>::put(grace_period_length);
            DilutionBound::<T>::put(dilution_bound);

            ProposalDeposit::<T>::put(proposal_deposit);
            ProcessingReward::<T>::put(processing_reward);
            let member = super::Member {
                shares: 1,
                highest_index_yes_vote: 0,
                loot: 0,
                jailed_at: 0,
                exists: true,
                delegate_key: who.clone(),
            };
            Members::<T>::insert(who.clone(), member);
            AddressOfDelegates::<T>::insert(who.clone(), who.clone());
            TotalShares::<T>::put(1);
            Self::deposit_event(Event::SummonComplete(who, 1));
            Ok(().into())
        }

        /// Anyone can submit proposal, but need to ensure enough tokens
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn submit_proposal(
            origin: OriginFor<T>,
            applicant: T::AccountId,
            #[pallet::compact] tribute_offered: BalanceOf<T>,
            shares_requested: u128,
            loot_requested: u128,
            #[pallet::compact] payment_requested: BalanceOf<T>,
            details: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            if Members::<T>::contains_key(who.clone()) {
                ensure!(
                    Members::<T>::get(who.clone()).unwrap().jailed_at == 0,
                    Error::<T>::MemberInJail
                );
            }
            let total_requested = loot_requested.checked_add(shares_requested).unwrap();
            let future_shares = TotalShares::<T>::get()
                .checked_add(total_requested)
                .unwrap();
            ensure!(
                future_shares <= T::MaxShares::get(),
                Error::<T>::SharesOverFlow
            );

            // collect proposal deposit from proposer and store it in the Moloch until the proposal is processed
            let _ =
                T::Currency::transfer(&who, &Self::custody_account(), tribute_offered, KeepAlive);

            let tribute_offered_num = Self::balance_to_u128(tribute_offered);
            let payment_requested_num = Self::balance_to_u128(payment_requested);
            let flags = [false; 6];
            Self::create_proposal(
                who.clone(),
                applicant.clone(),
                shares_requested,
                loot_requested,
                tribute_offered_num,
                payment_requested_num,
                details,
                flags,
            );
            Ok(().into())
        }

        /// propose a guild kick proposal
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn submit_guild_kick_proposal(
            origin: OriginFor<T>,
            member_to_kick: T::AccountId,
            details: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                Members::<T>::contains_key(member_to_kick.clone()),
                Error::<T>::NotMember
            );
            let member = Members::<T>::get(member_to_kick.clone());
            ensure!(
                member.clone().unwrap().shares > 0 || member.clone().unwrap().loot > 0,
                Error::<T>::NoEnoughShares
            );
            ensure!(member.clone().unwrap().jailed_at == 0, Error::<T>::MemberInJail);

            // [sponsored, processed, didPass, cancelled, whitelist, guildkick]
            let mut flags = [false; 6];
            flags[5] = true;
            Self::create_proposal(
                who.clone(),
                member_to_kick.clone(),
                0,
                0,
                0,
                0,
                details,
                flags,
            );
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn sponsor_proposal(
            origin: OriginFor<T>,
            proposal_index: u128,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                Members::<T>::contains_key(who.clone()),
                Error::<T>::NotMember
            );
            ensure!(
                Proposals::<T>::contains_key(proposal_index),
                Error::<T>::ProposalNotExist
            );
            let proposal = Proposals::<T>::get(proposal_index);
            // check proposal status
            ensure!(!proposal.clone().unwrap().flags[0], Error::<T>::ProposalHasSponsored);
            ensure!(!proposal.clone().unwrap().flags[3], Error::<T>::ProposalHasAborted);
            // reject in jailed memeber to process
            if Members::<T>::contains_key(who.clone()) {
                ensure!(
                    Members::<T>::get(who.clone()).unwrap().jailed_at == 0,
                    Error::<T>::MemberInJail
                );
            }

            // collect proposal deposit from proposer and store it in the Moloch until the proposal is processed
            let _ = T::Currency::transfer(
                &who,
                &Self::account_id(),
                ProposalDeposit::<T>::get(),
                KeepAlive,
            );

            if proposal.clone().unwrap().flags[5] {
                ensure!(
                    !ProsedToKick::<T>::contains_key(proposal.clone().unwrap().applicant.clone()),
                    Error::<T>::MemberInJail
                );
                ProsedToKick::<T>::insert(proposal.clone().unwrap().applicant, true);
            }
            let proposal_queue = ProposalQueue::<T>::get();
            let proposal_period = match proposal_queue.len() {
                0 => 0,
                n => Proposals::<T>::get(proposal_queue[n - 1]).clone().unwrap().starting_period,
            };
            let starting_period = proposal_period
                .max(Self::get_current_period())
                .checked_add(1)
                .unwrap();
            Proposals::<T>::mutate(proposal_index, |p| {
                p.clone().unwrap().starting_period = starting_period;
                // sponsored
                p.clone().unwrap().flags[0] = true;
                p.clone().unwrap().sponsor = AddressOfDelegates::<T>::get(who.clone()).clone().unwrap();
            });
            ProposalQueue::<T>::append(proposal_index);

            Ok(().into())
        }

        /// One of the members submit a vote
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn submit_vote(
            origin: OriginFor<T>,
            proposal_index: u128,
            vote_unit: u8,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                AddressOfDelegates::<T>::contains_key(who.clone()),
                Error::<T>::NotMember
            );
            let delegate = AddressOfDelegates::<T>::get(who.clone());
            let member = Members::<T>::get(delegate.clone().unwrap().clone());
            ensure!(member.clone().unwrap().shares > 0, Error::<T>::NoEnoughShares);

            let proposal_len = ProposalQueue::<T>::get().len();
            ensure!(
                proposal_index < proposal_len.try_into().unwrap(),
                Error::<T>::ProposalNotExist
            );
            let _usize_proposal_index = TryInto::<usize>::try_into(proposal_index).ok().unwrap();
            let proposal_id = ProposalQueue::<T>::get()[_usize_proposal_index];
            let proposal = Proposals::<T>::get(proposal_id);
            ensure!(vote_unit < 3 && vote_unit > 0, Error::<T>::InvalidVote);
            ensure!(
                Self::get_current_period() >= proposal.clone().unwrap().starting_period,
                Error::<T>::ProposalNotStart
            );
            ensure!(
                Self::get_current_period()
                    < VotingPeriodLength::<T>::get() + proposal.clone().unwrap().starting_period,
                Error::<T>::ProposalExpired
            );
            ensure!(
                !ProposalVotes::<T>::contains_key(proposal_index, delegate.clone().unwrap().clone()),
                Error::<T>::MemberHasVoted
            );
            ensure!(!proposal.clone().unwrap().flags[3], Error::<T>::ProposalHasAborted);
            let vote = match vote_unit {
                1 => Vote::Yes,
                2 => Vote::No,
                _ => Vote::Null,
            };
            ProposalVotes::<T>::insert(proposal_id, delegate.clone().unwrap().clone(), vote_unit);

            // update proposal
            Proposals::<T>::mutate(proposal_id, |p| {
                if vote == Vote::Yes {
                    p.clone().unwrap().yes_votes = proposal.clone().unwrap().yes_votes.checked_add(member.clone().unwrap().shares).unwrap();
                    if proposal_index > member.unwrap().highest_index_yes_vote {
                        Members::<T>::mutate(delegate.clone().unwrap().clone(), |mem| {
                            mem.clone().unwrap().highest_index_yes_vote = proposal_index;
                        });
                    }

                    // update max yes
                    let all_loot_shares = TotalShares::<T>::get()
                        .checked_add(TotalLoot::<T>::get())
                        .unwrap();
                    if all_loot_shares > proposal.clone().unwrap().max_total_shares_at_yes {
                        p.clone().unwrap().max_total_shares_at_yes = all_loot_shares;
                    }
                } else if vote == Vote::No {
                    p.clone().unwrap().no_votes = proposal.clone().unwrap().no_votes.checked_add(member.clone().unwrap().shares).unwrap();
                }
            });
            Self::deposit_event(Event::SubmitVote(proposal_index, who, delegate.clone().unwrap(), vote_unit));
            Ok(().into())
        }

        /// Process a proposal in queue
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn process_proposal(
            origin: OriginFor<T>,
            proposal_index: u128,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let proposal_len = ProposalQueue::<T>::get().len();
            ensure!(
                proposal_index < proposal_len.try_into().unwrap(),
                Error::<T>::ProposalNotExist
            );
            let _usize_proposal_index = TryInto::<usize>::try_into(proposal_index).ok().unwrap();
            let proposal_id = ProposalQueue::<T>::get()[_usize_proposal_index];
            let proposal = &mut Proposals::<T>::get(proposal_id);
            ensure!(
                !proposal.clone().unwrap().flags[4] && !proposal.clone().unwrap().flags[5],
                Error::<T>::NotStandardProposal
            );
            ensure!(
                Self::get_current_period()
                    - VotingPeriodLength::<T>::get()
                    - GracePeriodLength::<T>::get()
                    >= proposal.clone().unwrap().starting_period,
                Error::<T>::ProposalNotReady
            );
            ensure!(proposal.clone().unwrap().flags[1] == false, Error::<T>::ProposalHasProcessed);
            ensure!(
                proposal_index == 0
                    || Proposals::<T>::get(ProposalQueue::<T>::get()[_usize_proposal_index - 1]).unwrap()
                        .flags[1],
                Error::<T>::PreviousProposalNotProcessed
            );

            proposal.clone().unwrap().flags[1] = true;
            let mut did_pass = Self::should_pass(Proposals::<T>::get(proposal_id).unwrap());
            let tribute_offered = Self::u128_to_balance(proposal.clone().unwrap().tribute_offered);
            let free_token_num =
                Self::balance_to_u128(T::Currency::free_balance(&Self::account_id()));
            // too many tokens requested
            if proposal.clone().unwrap().payment_requested > free_token_num {
                did_pass = false;
            }
            // shares+loot overflow
            let total_requested = proposal.clone().unwrap()
                .loot_requested
                .checked_add(proposal.clone().unwrap().shares_requested)
                .unwrap();
            let future_shares = TotalShares::<T>::get()
                .checked_add(total_requested)
                .unwrap();
            ensure!(
                future_shares.checked_add(TotalLoot::<T>::get()).unwrap() <= T::MaxShares::get(),
                Error::<T>::SharesOverFlow
            );

            // TODO: guild is full

            // Proposal passed
            if did_pass {
                // mark did_pass to true
                proposal.clone().unwrap().flags[2] = true;

                // if the applicant is already a member, add to their existing shares
                if Members::<T>::contains_key(&proposal.clone().unwrap().applicant) {
                    Members::<T>::mutate(&proposal.clone().unwrap().applicant, |mem| {
                        mem.clone().unwrap().shares = mem.clone().unwrap().shares.checked_add(proposal.clone().unwrap().shares_requested).unwrap();
                        mem.clone().unwrap().loot = mem.clone().unwrap().loot.checked_add(proposal.clone().unwrap().loot_requested).unwrap();
                    });
                } else {
                    // if the applicant address is already taken by a member's delegateKey, reset it to their member address
                    if AddressOfDelegates::<T>::contains_key(proposal.clone().unwrap().applicant.clone()) {
                        let delegate = AddressOfDelegates::<T>::get(proposal.clone().unwrap().applicant.clone());
                        Members::<T>::mutate(delegate.clone().unwrap().clone(), |mem| {
                            mem.clone().unwrap().delegate_key = delegate.clone().unwrap().clone();
                        });
                        AddressOfDelegates::<T>::insert(delegate.clone().unwrap().clone(), delegate.clone().unwrap().clone());
                    }
                    // add new member
                    let member = super::Member {
                        shares: proposal.clone().unwrap().shares_requested,
                        highest_index_yes_vote: 0,
                        loot: proposal.clone().unwrap().loot_requested,
                        jailed_at: 0,
                        exists: true,
                        delegate_key: proposal.clone().unwrap().applicant.clone(),
                    };
                    Members::<T>::insert(proposal.clone().unwrap().applicant.clone(), member);
                    AddressOfDelegates::<T>::insert(
                        proposal.clone().unwrap().applicant.clone(),
                        proposal.clone().unwrap().applicant.clone(),
                    );
                }

                // mint new shares
                let totoal_shares = TotalShares::<T>::get()
                    .checked_add(proposal.clone().unwrap().shares_requested)
                    .unwrap();
                TotalShares::<T>::put(totoal_shares);
                // transfer correponding balance from custody account to guild bank's free balance
                let _res = T::Currency::transfer(
                    &Self::custody_account(),
                    &Self::account_id(),
                    tribute_offered,
                    AllowDeath,
                );
            } else {
                // Proposal failed
                // return the balance of applicant
                let _ = T::Currency::transfer(
                    &Self::custody_account(),
                    &proposal.clone().unwrap().applicant,
                    tribute_offered,
                    AllowDeath,
                );
            }

            // need to mutate for update
            Proposals::<T>::insert(proposal_id, proposal.clone().unwrap().clone());

            // send reward
            let _ = T::Currency::transfer(
                &Self::account_id(),
                &who,
                ProcessingReward::<T>::get(),
                KeepAlive,
            );
            // return deposit with reward slashed
            let rest_balance = ProposalDeposit::<T>::get() - ProcessingReward::<T>::get();
            let _ = T::Currency::transfer(
                &Self::account_id(),
                &proposal.clone().unwrap().proposer,
                rest_balance,
                KeepAlive,
            );

            Self::deposit_event(Event::ProcessProposal(
                proposal_index,
                proposal.clone().unwrap().applicant.clone(),
                proposal.clone().unwrap().proposer.clone(),
                proposal.clone().unwrap().tribute_offered,
                proposal.clone().unwrap().shares_requested,
                did_pass,
            ));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn process_guild_kick_proposal(
            origin: OriginFor<T>,
            proposal_index: u128,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let proposal_len = ProposalQueue::<T>::get().len();
            ensure!(
                proposal_index < proposal_len.try_into().unwrap(),
                Error::<T>::ProposalNotExist
            );
            let _usize_proposal_index = TryInto::<usize>::try_into(proposal_index).ok().unwrap();
            let proposal_id = ProposalQueue::<T>::get()[_usize_proposal_index];
            let proposal = &mut Proposals::<T>::get(proposal_id);
            // ensure guild kick proposal
            ensure!(proposal.clone().unwrap().flags[5], Error::<T>::NotKickProposal);
            ensure!(
                Self::get_current_period()
                    - VotingPeriodLength::<T>::get()
                    - GracePeriodLength::<T>::get()
                    >= proposal.clone().unwrap().starting_period,
                Error::<T>::ProposalNotReady
            );
            ensure!(proposal.clone().unwrap().flags[1] == false, Error::<T>::ProposalHasProcessed);
            ensure!(
                proposal_index == 0
                    || Proposals::<T>::get(ProposalQueue::<T>::get()[_usize_proposal_index - 1]).unwrap()
                        .flags[1],
                Error::<T>::PreviousProposalNotProcessed
            );

            proposal.clone().unwrap().flags[1] = true;
            let did_pass = Self::should_pass(Proposals::<T>::get(proposal_id).unwrap());
            if did_pass {
                // mark did_pass to true
                proposal.clone().unwrap().flags[2] = true;
                // update memeber status, i.e. jailed and slash shares
                Members::<T>::mutate(proposal.clone().unwrap().applicant.clone(), |member| {
                    member.clone().unwrap().jailed_at = proposal_index;
                    member.clone().unwrap().loot = member.clone().unwrap().loot.checked_add(member.clone().unwrap().shares).unwrap();
                    let total_shares = TotalShares::<T>::get().checked_sub(member.clone().unwrap().shares).unwrap();
                    let total_loot = TotalLoot::<T>::get().checked_add(member.clone().unwrap().shares).unwrap();
                    TotalLoot::<T>::put(total_loot);
                    TotalShares::<T>::put(total_shares);
                    member.clone().unwrap().shares = 0;
                });
            }

            ProsedToKick::<T>::insert(proposal.clone().unwrap().applicant.clone(), false);

            // send reward
            let _ = T::Currency::transfer(
                &Self::account_id(),
                &who,
                ProcessingReward::<T>::get(),
                KeepAlive,
            );
            // return deposit with reward slashed
            let rest_balance = ProposalDeposit::<T>::get() - ProcessingReward::<T>::get();
            let _ = T::Currency::transfer(
                &Self::account_id(),
                &proposal.clone().unwrap().proposer,
                rest_balance,
                KeepAlive,
            );

            Ok(().into())
        }

        /// proposer abort a proposal
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn abort(origin: OriginFor<T>, proposal_index: u128) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                Proposals::<T>::contains_key(proposal_index),
                Error::<T>::ProposalNotExist
            );
            let proposal = &mut Proposals::<T>::get(proposal_index);
            ensure!(who == proposal.clone().unwrap().proposer, Error::<T>::NotProposalProposer);
            ensure!(!proposal.clone().unwrap().flags[0], Error::<T>::ProposalHasSponsored);
            ensure!(!proposal.clone().unwrap().flags[3], Error::<T>::ProposalHasAborted);
            let token_to_abort = proposal.clone().unwrap().tribute_offered;
            proposal.clone().unwrap().tribute_offered = 0;
            proposal.clone().unwrap().flags[3] = true;

            // need to mutate for update
            Proposals::<T>::insert(proposal_index, proposal.clone().unwrap().clone());
            // return the token to applicant and delete record
            let _ = T::Currency::transfer(
                &Self::custody_account(),
                &proposal.clone().unwrap().proposer,
                Self::u128_to_balance(token_to_abort),
                AllowDeath,
            );

            Self::deposit_event(Event::Abort(proposal_index, who.clone()));
            Ok(().into())
        }

        /// Member rage quit
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn rage_quit(
            origin: OriginFor<T>,
            shares_to_burn: u128,
            loot_to_burn: u128,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::member_quit(who, shares_to_burn, loot_to_burn)
        }

        /// kick anymember  in jail
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn rage_kick(
            origin: OriginFor<T>,
            member_to_kick: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;
            let member = Members::<T>::get(member_to_kick.clone());
            ensure!(member.clone().unwrap().jailed_at != 0, Error::<T>::MemberNotInJail);
            ensure!(member.clone().unwrap().loot > 0, Error::<T>::NoEnoughLoot);
            Self::member_quit(member_to_kick, 0, member.clone().unwrap().loot)
        }

        /// update the delegate
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn update_delegate(
            origin: OriginFor<T>,
            delegate_key: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // skip checks if member is setting the delegate key to their member address
            if who != delegate_key {
                ensure!(
                    !Members::<T>::contains_key(delegate_key.clone()),
                    Error::<T>::NoOverwriteMember
                );
                let delegate = AddressOfDelegates::<T>::get(delegate_key.clone());
                ensure!(
                    !Members::<T>::contains_key(delegate.clone().unwrap().clone()),
                    Error::<T>::NoOverwriteDelegate
                );
            }

            let member = &mut Members::<T>::get(who.clone());
            AddressOfDelegates::<T>::remove(member.clone().unwrap().delegate_key.clone());
            AddressOfDelegates::<T>::insert(delegate_key.clone(), who.clone());
            member.clone().unwrap().delegate_key = delegate_key.clone();
            Self::deposit_event(Event::UpdateDelegateKey(who, delegate_key));
            Ok(().into())
        }
    }

    //** Our helper functions.**//

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        pub fn custody_account() -> T::AccountId {
            T::PalletId::get().into_sub_account("custody")
        }

        pub fn u128_to_balance(cost: u128) -> BalanceOf<T> {
            TryInto::<BalanceOf<T>>::try_into(cost).ok().unwrap()
        }

        pub fn balance_to_u128(balance: BalanceOf<T>) -> u128 {
            TryInto::<u128>::try_into(balance).ok().unwrap()
        }

        pub fn get_current_period() -> u128 {
            let now = TryInto::<u128>::try_into(pallet_timestamp::Pallet::<T>::now())
                .ok()
                .unwrap();
            let summon_time = TryInto::<u128>::try_into(SummonTime::<T>::get())
                .ok()
                .unwrap();
            let diff = now.checked_sub(summon_time).unwrap();
            // the timestamp is in milli seconds
            diff.checked_div(1000)
                .unwrap()
                .checked_div(PeriodDuration::<T>::get().into())
                .unwrap()
        }

        pub fn create_proposal(
            proposer: T::AccountId,
            applicant: T::AccountId,
            shares_requested: u128,
            loot_requested: u128,
            tribute_offered: u128,
            payment_requested: u128,
            details: Vec<u8>,
            flags: [bool; 6],
        ) {
            let proposal_index = ProposalCount::<T>::get();
            let proposal = Proposal {
                proposer: proposer.clone(),
                applicant: applicant.clone(),
                sponsor: proposer.clone(),
                shares_requested: shares_requested,
                starting_period: 0,
                yes_votes: 0,
                no_votes: 0,
                details: details,
                max_total_shares_at_yes: 0,
                loot_requested: loot_requested,
                tribute_offered: tribute_offered,
                payment_requested: payment_requested,
                flags: flags,
            };
            Proposals::<T>::insert(proposal_index, proposal);
            Self::deposit_event(Event::SubmitProposal(
                proposal_index,
                proposer.clone(),
                proposer,
                applicant,
                tribute_offered,
                shares_requested,
            ));
            ProposalCount::<T>::put(proposal_index + 1);
        }

        pub fn should_pass(proposal: ProposalOf<T>) -> bool {
            let mut pass = proposal.yes_votes > proposal.no_votes;
            // as anyone can process the proposal and get rewarded, so do not fail here
            if TotalShares::<T>::get()
                .checked_mul(DilutionBound::<T>::get())
                .unwrap()
                < proposal.max_total_shares_at_yes
            {
                Self::deposit_event(Event::DilutionBoundExeceeds(
                    TotalShares::<T>::get(),
                    DilutionBound::<T>::get(),
                    proposal.max_total_shares_at_yes,
                ));
                pass = false;
            }

            if Members::<T>::get(proposal.applicant.clone()).unwrap().jailed_at != 0 {
                pass = false;
            }
            pass
        }

        pub fn member_quit(
            who: T::AccountId,
            shares_to_burn: u128,
            loot_to_burn: u128,
        ) -> frame_support::dispatch::DispatchResultWithPostInfo {
            use frame_support::ensure;

            ensure!(
                Members::<T>::contains_key(who.clone()),
                Error::<T>::NotMember
            );
            let member = Members::<T>::get(who.clone());
            ensure!(member.clone().unwrap().shares >= shares_to_burn, Error::<T>::NoEnoughShares);
            // check if can rage quit
            let proposal_index = member.clone().unwrap().highest_index_yes_vote;
            ensure!(
                proposal_index < ProposalQueue::<T>::get().len().try_into().unwrap(),
                Error::<T>::ProposalNotExist
            );
            let _usize_proposal_index = TryInto::<usize>::try_into(proposal_index).ok().unwrap();
            let proposal_id = ProposalQueue::<T>::get()[_usize_proposal_index];
            let proposal = Proposals::<T>::get(proposal_id);
            ensure!(proposal.unwrap().flags[1], Error::<T>::ProposalNotProcessed);

            // burn shares and loot
            Members::<T>::mutate(who.clone(), |mem| {
                mem.clone().unwrap().shares = member.clone().unwrap().shares.checked_sub(shares_to_burn).unwrap();
                mem.clone().unwrap().loot = member.clone().unwrap().loot.checked_sub(loot_to_burn).unwrap();
            });
            let initial_total = TotalShares::<T>::get()
                .checked_add(TotalLoot::<T>::get())
                .unwrap();
            let total_to_burn = shares_to_burn.checked_add(loot_to_burn).unwrap();
            let rest_shares = TotalShares::<T>::get().checked_sub(shares_to_burn).unwrap();
            TotalShares::<T>::put(rest_shares);
            let rest_loot = TotalLoot::<T>::get().checked_sub(loot_to_burn).unwrap();
            TotalLoot::<T>::put(rest_loot);

            // withdraw the tokens
            let amount = Self::balance_to_u128(T::Currency::free_balance(&Self::account_id()));
            let balance = amount
                .checked_mul(total_to_burn)
                .unwrap()
                .checked_div(initial_total)
                .unwrap();
            let _ = T::Currency::transfer(
                &Self::account_id(),
                &who,
                Self::u128_to_balance(balance),
                KeepAlive,
            );

            Self::deposit_event(Event::Ragequit(who.clone(), shares_to_burn));
            Ok(().into())
        }
    }
}
