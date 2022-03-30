#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::{AccountIdConversion, AtLeast32BitUnsigned, BlockNumberProvider, Saturating},
        Perbill, SaturatedConversion,
    };
    use sp_std::{prelude::*, vec::Vec};

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // DoraFactory Crowdloan rewards pallet
    #[pallet::pallet]
    #[pallet::without_storage_info]
    // The crowdloan rewards pallet
    pub struct Pallet<T>(PhantomData<T>);

    pub const PALLET_ID: PalletId = PalletId(*b"DoraRewa");

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        // tracking the vesting process
        type VestingBlockNumber: AtLeast32BitUnsigned + Parameter + Default + Into<BalanceOf<Self>>;

        // get the blocknumber by this provider
        type VestingBlockProvider: BlockNumberProvider<BlockNumber = Self::VestingBlockNumber>;

        // the first reward percentage of total reward
        type FirstVestPercentage: Get<Perbill>;

        // this parameter control the max contributor list length
        #[pallet::constant]
        type MaxContributorsNumber: Get<u32>;
    }

    //
    // record the contributor's reward info
    //
    #[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct RewardInfo<T: Config> {
        pub total_reward: BalanceOf<T>,
        pub claimed_reward: BalanceOf<T>,
        pub track_block_number: T::VestingBlockNumber,
    }

    #[pallet::storage]
    #[pallet::storage_prefix = "InitBlock"]
    #[pallet::getter(fn init_vesting_block)]
    /// Vesting block height at the initialization of the pallet
    type InitVestingBlock<T: Config> = StorageValue<_, T::VestingBlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::storage_prefix = "EndBlock"]
    #[pallet::getter(fn end_vesting_block)]
    /// Vesting block height at the initialization of the pallet
    type EndVestingBlock<T: Config> = StorageValue<_, T::VestingBlockNumber, ValueQuery>;

    // record contributor's info (total reward, claimed reward, claim linear block track)
    #[pallet::storage]
    #[pallet::getter(fn rewards_info)]
    type ContributorsInfo<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, RewardInfo<T>, OptionQuery>;

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        // invalid account (not exist in contributor list)
        NotInContributorList,
        // No setting in Ending lease block
        NotSettingEndingLeaseBlock,
        // Ending lease block setting error
        InvalidEndingLeaseBlock,
        // claimed all the reward
        NoLeftRewards,
        // exceed the contributor number
        TooManyContributors,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // update contributor's reward info(accountId, claimed reward, left reward)
        UpdateContributorsInfo(T::AccountId, BalanceOf<T>, BalanceOf<T>),
        // distribute Vest <source account, destination account, amount>
        DistributeReward(T::AccountId, T::AccountId, BalanceOf<T>),
        // set end lease block
        EndleasingBlock(T::VestingBlockNumber),
    }

    //
    // Question: how many accounts can be distributed in one block, that is very important.
    // i think we can set four accounts in one block to test.
    //
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: <T as frame_system::Config>::BlockNumber) {
            // record the block number in relaychain when our parachain launch it.
            // this time means our reward is starting...
            if n == 1u32.into() {
                <InitVestingBlock<T>>::put(T::VestingBlockProvider::current_block_number());
            }
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The amount of funds this pallet controls
        pub funded_amount: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                funded_amount: 1u32.into(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        // This sets the pre-funds of this Reward pallet
        fn build(&self) {
            T::Currency::deposit_creating(&Pallet::<T>::account_id(), self.funded_amount);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        //
        // provid contributors claim for their rewards
        //
        #[pallet::weight(0)]
        pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // check current acccount is in the contributor list ?
            // if exist, get his reward info
            let contribute_info =
                <ContributorsInfo<T>>::get(who.clone()).ok_or(Error::<T>::NotInContributorList)?;

            // ensure we have set the ending lease block
            ensure!(
                <EndVestingBlock<T>>::get() != 0u32.into(),
                <Error<T>>::NotSettingEndingLeaseBlock,
            );

            // if contributor's claimed reward reach his total reward, don't send DORA
            ensure!(
                contribute_info.claimed_reward < contribute_info.total_reward,
                <Error<T>>::NoLeftRewards,
            );

            // compute the total linear reward block(ending lease - start lease)
            let total_reward_period = <EndVestingBlock<T>>::get() - <InitVestingBlock<T>>::get();

            // if current block height bigger than ending lease block ? this operation need??
            let now =
                if T::VestingBlockProvider::current_block_number() >= <EndVestingBlock<T>>::get() {
                    <EndVestingBlock<T>>::get()
                } else {
                    T::VestingBlockProvider::current_block_number()
                };

            // compute the fist reward with total reward by the percentage
            let first_reward = T::FirstVestPercentage::get() * contribute_info.total_reward;

            let left_linear_reward = contribute_info.total_reward - first_reward;

            let curr_linear_reward_period = now
                .clone()
                .saturating_sub(contribute_info.track_block_number.clone());
            let current_linear_reward = left_linear_reward
                .saturating_mul(curr_linear_reward_period.into())
                / total_reward_period.into();

            // Get the current left reward
            let coming_reward = if contribute_info.claimed_reward == 0u32.into() {
                // if current user never claim the rewards, distribute `fisrt reward` + `current
                // linear block reward`. Get the linear reward block number from the first block to
                // current block

                // update the claimed reward and track block number
                let new_contribute_info = RewardInfo {
                    total_reward: contribute_info.total_reward,
                    claimed_reward: first_reward + current_linear_reward,
                    track_block_number: now.clone(),
                };
                <ContributorsInfo<T>>::insert(who.clone(), new_contribute_info);
                Self::deposit_event(<Event<T>>::UpdateContributorsInfo(
                    who.clone(),
                    contribute_info.total_reward,
                    first_reward + current_linear_reward,
                ));
                first_reward + current_linear_reward
            } else {
                // if current user have got some rewards, but the lease is not ending, get the
                // latest linear block reward compute by the block period: now block number - last
                // track block number

                // if reach or higher the end lease block, the claimed reward < total reward, distribute the left reward
                if contribute_info.track_block_number >= <EndVestingBlock<T>>::get() {
                    contribute_info.total_reward - contribute_info.claimed_reward
                } else {
                    let new_contribute_info = RewardInfo {
                        total_reward: contribute_info.total_reward,
                        claimed_reward: contribute_info.claimed_reward + current_linear_reward,
                        track_block_number: now.clone(),
                    };
                    <ContributorsInfo<T>>::insert(who.clone(), new_contribute_info);
                    Self::deposit_event(<Event<T>>::UpdateContributorsInfo(
                        who.clone(),
                        contribute_info.total_reward,
                        contribute_info.claimed_reward + current_linear_reward,
                    ));
                    current_linear_reward
                }
            };

            // distribute current reward to contributor
            Self::distribute_to_contributors(who.clone(), coming_reward.saturated_into::<u128>())?;
            Self::deposit_event(<Event<T>>::DistributeReward(
                Self::account_id(),
                who.clone(),
                coming_reward.saturated_into::<BalanceOf<T>>(),
            ));
            Ok(().into())
        }

        ///
        ///  step 1:
        ///  set a contributors rewards info
        ///  this operation should be execute by sudo user
        #[pallet::weight(0)]
        pub fn initialize_contributors_list(
            origin: OriginFor<T>,
            //import contributor list
            contributor_list: Vec<(T::AccountId, BalanceOf<T>)>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            // ensure don;t exceed contributor list length
            ensure!(
                contributor_list.len() as u32 <= T::MaxContributorsNumber::get(),
                <Error<T>>::TooManyContributors,
            );
            // update the contributors list
            for (contributor_account, contribution_value) in &contributor_list {
                // compute contributor's total rewards
                let total_reward = (contribution_value.saturating_mul(3u32.into()))
                    .saturated_into::<BalanceOf<T>>();
                // initialize the contrbutor's rewards info
                log::info!("Initializing block is :{:?}", <InitVestingBlock<T>>::get());
                let reward_info = RewardInfo {
                    total_reward,
                    claimed_reward: 0u128.saturated_into::<BalanceOf<T>>(),
                    track_block_number: <InitVestingBlock<T>>::get(),
                };
                <ContributorsInfo<T>>::insert(contributor_account.clone(), reward_info.clone());
                Self::deposit_event(Event::UpdateContributorsInfo(
                    contributor_account.clone(),
                    total_reward,
                    0u128.saturated_into::<BalanceOf<T>>(),
                ));
            }

            Ok(().into())
        }

        //
        //  step2:
        // 	check the lease ending block
        //
        #[pallet::weight(0)]
        pub fn complete_initialization(
            origin: OriginFor<T>,
            lease_ending_block: T::VestingBlockNumber,
        ) -> DispatchResult {
            ensure_root(origin)?;
            // ending lease block should higher than the init lease block, invalid setting will cause overflow
            ensure!(
                lease_ending_block > <InitVestingBlock<T>>::get(),
                <Error<T>>::InvalidEndingLeaseBlock,
            );

            <EndVestingBlock<T>>::put(lease_ending_block.clone());
            Self::deposit_event(<Event<T>>::EndleasingBlock(lease_ending_block.clone()));

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The account ID of the treasury pot.
        ///
        /// This actually does computation. If you need to keep using it, then make sure you cache
        /// the value and only call this once.
        pub fn account_id() -> T::AccountId {
            PALLET_ID.into_account()
        }

        // distributed by Pallet account
        pub fn distribute_to_contributors(
            contributor_account: T::AccountId,
            value: u128,
        ) -> DispatchResult {
            T::Currency::transfer(
                &Self::account_id(),
                &contributor_account,
                value.saturated_into(),
                ExistenceRequirement::AllowDeath,
            )?;
            Ok(().into())
        }
    }
}
