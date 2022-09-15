#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod weights;

pub mod xcm_config;

pub mod constants;

use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, Convert},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        ConstBool, ConstU32, Contains, Currency, EitherOfDiverse, EqualPrivilegeOnly, Everything,
        Imbalance, OnUnbalanced,
    },
    weights::{
        constants::WEIGHT_PER_SECOND, ConstantMultiplier, DispatchClass, Weight,
        WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    },
    PalletId,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};
use xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot Imports
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

use weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight};

// XCM Imports
use xcm::latest::prelude::BodyId;
use xcm_executor::XcmExecutor;

// ORML Imports
use orml_currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;

pub use primitives::{
    AccountId, Address, Amount, Balance, BlockNumber, CurrencyId, Hash, Index, ReserveIdentifier,
    Signature, CENTS, DOLLARS, EXISTENTIAL_DEPOSIT, MICROUNIT, MILLICENTS, MILLIUNIT, UNIT,
};

pub use constants::time::*;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    use sp_runtime::{generic, traits::BlakeTwo256};

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("DORA KSM Parachain"),
    impl_name: create_runtime_str!("DORA KSM Parachain"),
    authoring_version: 1,
    spec_version: 30,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

pub fn ksm_per_second() -> u128 {
    let base_weight = Balance::from(ExtrinsicBaseWeight::get());
    let base_tx_fee = DOLLARS / 1000;
    let base_tx_per_second = (WEIGHT_PER_SECOND as u128) / base_weight;
    let fee_per_second = base_tx_per_second * base_tx_fee;
    fee_per_second / 100
}

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;

    // This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
    //  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
    // `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
    // the lazy contract deletion.
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u16 = 128;
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
    pub const TreasuryPalletId: PalletId = PalletId(*b"dora/try");
}

// Configure FRAME pallets to include in runtime.
impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = Everything;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Runtime version.
    type Version = Version;
    /// Converts a module to an index of this module in the runtime.
    type PalletInfo = PalletInfo;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// The action to take on a Runtime Upgrade
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = (CollatorSelection,);
}

parameter_types! {
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    /// The type for recording an account's balance.
    type Balance = Balance;
    type DustRemoval = ();
    /// The ubiquitous event type.
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = ReserveIdentifier;
}

parameter_types! {
    /// Relay Chain `TransactionByteFee` / 10
    pub const TransactionByteFee: Balance = 10 * MICROUNIT;
    pub const OperationalFeeMultiplier: u8 = 5;
}

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort off change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // in dora, 1 weight = 0.8 balance
        let p = MILLIUNIT / 10;
        let q = Balance::from(ExtrinsicBaseWeight::get());

        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(mut fees) = fees_then_tips.next() {
            if let Some(tips) = fees_then_tips.next() {
                tips.merge_into(&mut fees);
            }
            // for fees and tips, 100% to treasury
            <ToTreasury as OnUnbalanced<_>>::on_unbalanced(fees);
        }
    }
}

impl pallet_transaction_payment::Config for Runtime {
    type Event = Event;
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightToFee = WeightToFee;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnSystemEvent = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type OutboundXcmpMessageSource = XcmpQueue;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
    type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = ();
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
    pub const Period: u32 = 6 * HOURS;
    pub const Offset: u32 = 0;
    pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    // we don't have stash and controller, thus we don't need the convert as well.
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = CollatorSelection;
    // Essentially just Aura, but lets be pedantic.
    type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type MaxAuthorities = MaxAuthorities;
    type DisabledValidators = ();
}

parameter_types! {
    pub const PotId: PalletId = PalletId(*b"PotStake");
    pub const MaxCandidates: u32 = 1000;
    pub const MinCandidates: u32 = 5;
    pub const SessionLength: BlockNumber = 6 * HOURS;
    pub const MaxInvulnerables: u32 = 100;
    pub const ExecutiveBody: BodyId = BodyId::Executive;
}

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type UpdateOrigin = CollatorSelectionUpdateOrigin;
    type PotId = PotId;
    type MaxCandidates = MaxCandidates;
    type MinCandidates = MinCandidates;
    type MaxInvulnerables = MaxInvulnerables;
    // should be a multiple of session or things will get inconsistent
    type KickThreshold = Period;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
    type ValidatorRegistration = Session;
    type WeightInfo = ();
}

// pub struct ToStakingPot;
// impl OnUnbalanced<NegativeImbalance> for ToStakingPot {
//     fn on_nonzero_unbalanced(amount: NegativeImbalance) {
//         let staking_pot = PotId::get().into_account_truncating();
//         Balances::resolve_creating(&staking_pot, amount);
//     }
// }

parameter_types! {
    pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

pub struct ToTreasury;
impl OnUnbalanced<NegativeImbalance> for ToTreasury {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        let staking_treasury = TreasuryAccount::get();
        Balances::resolve_creating(&staking_treasury, amount);
    }
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

// Define the types required by the Scheduler pallet.
parameter_types! {
    pub MaximumSchedulerWeight: Weight = 10_000_000;
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u32> = Some(5 * MINUTES);
}

// Configure the runtime's implementation of the Scheduler pallet.
impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
    type PreimageProvider = Preimage;
    type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
    pub PreimageBaseDeposit: Balance = deposit(10, 64);
    pub PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
    type Event = Event;
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    // Max size 4MB allowed: 4096 * 1024
    type MaxSize = ConstU32<4_194_304>;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

pub type GeneralCouncilInstance = pallet_collective::Instance1;
pub type TechnicalCommitteeInstance = pallet_collective::Instance2;

pub type GeneralCouncilMembershipInstance = pallet_membership::Instance1;
pub type TechnicalCommitteeMembershipInstance = pallet_membership::Instance2;

// General Council
pub type EnsureRootOrAllGeneralCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, GeneralCouncilInstance, 1, 1>,
>;

pub type EnsureRootOrThreeFivethsGeneralCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, GeneralCouncilInstance, 3, 5>,
>;

pub type EnsureRootOrFourFivethsGeneralCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, GeneralCouncilInstance, 4, 5>,
>;

pub type EnsureRootOrHalfGeneralCouncil = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, GeneralCouncilInstance, 1, 2>,
>;

// Technical Committee Council
pub type EnsureRootOrAllTechnicalCommittee = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 1, 1>,
>;

pub type EnsureRootOrTwoThirdsTechnicalCommittee = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 2, 3>,
>;

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 2 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TechnicalCommitteeMotionDuration: BlockNumber = 1 * DAYS;
}

impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = TechnicalCommitteeMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = ();
}

impl pallet_membership::Config<TechnicalCommitteeMembershipInstance> for Runtime {
    type Event = Event;
    type AddOrigin = EnsureRootOrThreeFivethsGeneralCouncil;
    type RemoveOrigin = EnsureRootOrThreeFivethsGeneralCouncil;
    type SwapOrigin = EnsureRootOrThreeFivethsGeneralCouncil;
    type ResetOrigin = EnsureRootOrThreeFivethsGeneralCouncil;
    type PrimeOrigin = EnsureRootOrThreeFivethsGeneralCouncil;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
    type MaxMembers = CouncilMaxMembers;
    type WeightInfo = ();
}

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 3 * DAYS;
    pub const VotingPeriod: BlockNumber = 7 * DAYS;
    pub const FastTrackVotingPeriod: BlockNumber = 2 * DAYS;
    pub MinimumDeposit: Balance = 100 * MILLICENTS;
    pub const EnactmentPeriod: BlockNumber = 12 * HOURS;
    pub const VoteLockingPeriod: BlockNumber = 3 * DAYS;
    pub const CooloffPeriod: BlockNumber = 7 * DAYS;
}

impl pallet_democracy::Config for Runtime {
    type Proposal = Call;
    type Event = Event;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type VoteLockingPeriod = VoteLockingPeriod;
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin = EnsureRootOrThreeFivethsGeneralCouncil;
    /// A majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin = EnsureRootOrThreeFivethsGeneralCouncil;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin = EnsureRootOrAllGeneralCouncil;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    type InstantOrigin = EnsureRootOrAllTechnicalCommittee;
    type InstantAllowed = ConstBool<true>;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    // To cancel a proposal which has been passed, 4/5 of the council must agree to it.
    type CancellationOrigin = EnsureRootOrFourFivethsGeneralCouncil;
    type BlacklistOrigin = EnsureRoot<AccountId>;
    // To cancel a proposal before it has been passed, the technical committee must be unanimous or
    // Root must agree.
    type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cooloff period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type OperationalPreimageOrigin =
        pallet_collective::EnsureMember<AccountId, GeneralCouncilInstance>;
    type Slash = ToTreasury;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = ConstU32<100>;
    type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
    type MaxProposals = ConstU32<100>;
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

// Configure the pallet-qf in pallets/quadratic-funding.
parameter_types! {
    // pow(10,12) => Unit, for easy fee control, we use pow(10,9)
    pub const VoteUnit: u128 = 1000000000;
    // The base of unit per vote, should be 1 Unit of token for each vote
    pub const NumberOfUnit: u128 = 1000;
    // The ratio of fee for each trans, final value should be FeeRatio/NumberOfUnit
    pub const FeeRatio: u128 = 6;
    pub const QuadraticFundingPalletId: PalletId = PalletId(*b"py/quafd");
    pub const NameMinLength: u32 = 3;
    pub const NameMaxLength: u32 = 32;
    pub const AppId: u8 = 1;
    // minimal number of units to reserve to get qualified to vote
    pub const ReserveUnit: u128 = 1000000000000;
}

/// Configure the pallet-qf in pallets/quadratic-funding.
impl pallet_qf::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MultiCurrency = Currencies;
    type PalletId = QuadraticFundingPalletId;
    // Origin who can control the round
    type AdminOrigin = EnsureRoot<AccountId>;
    // Use the UnitOfVote from the parameter_types block.
    type UnitOfVote = VoteUnit;
    // Use the MinNickLength from the parameter_types block.
    type NumberOfUnitPerVote = NumberOfUnit;
    // Use the FeeRatio from the parameter_types block.
    type FeeRatioPerVote = FeeRatio;
    // The minimum length of project name
    type NameMinLength = NameMinLength;
    // The maximum length of project name
    type NameMaxLength = NameMaxLength;
    type ReserveUnit = ReserveUnit;
    type WeightInfo = pallet_qf::weights::DoraWeight<Runtime>;
}

parameter_types! {
    pub const DaoCorePalletId: PalletId = PalletId(*b"py/dcore");
    pub const TaxPercentNum: u32 = 3;
}

/// Configure the pallet-qf in pallets/quadratic-funding.
impl dao_core::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = pallet_balances::Pallet<Runtime>;
    type PalletId = DaoCorePalletId;
    type TaxInPercent = TaxPercentNum;
    type SupervisorOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
    pub const FirstVestPercentage: Perbill = Perbill::from_percent(20);
    pub const MaxContributorsNumber: u32 = 400;
}

impl pallet_dora_rewards::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Initialized = ConstBool<false>;
    type VestingBlockNumber = cumulus_primitives_core::relay_chain::BlockNumber;
    type VestingBlockProvider =
        cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
    type FirstVestPercentage = FirstVestPercentage;
    type MaxContributorsNumber = MaxContributorsNumber;
    type WeightInfo = pallet_dora_rewards::weights::DoraWeight<Runtime>;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        // every currency has a zero existential deposit
        match currency_id {
            _ => 0,
        }
    };
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
    vec![
        PotId::get().into_account_truncating(),
        TreasuryPalletId::get().into_account_truncating(),
    ]
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
    fn contains(a: &AccountId) -> bool {
        get_all_module_accounts().contains(a)
    }
}

impl orml_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = ();
    // type OnDust = orml_tokens::TransferDust<Runtime, NativeTreasuryAccount>;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = ReserveIdentifier;
    type DustRemovalWhitelist = DustRemovalWhitelist;
    type OnNewTokenAccount = ();
    type OnKilledTokenAccount = ();
}

impl orml_xcm::Config for Runtime {
    type Event = Event;
    type SovereignOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::DORA;
}

impl orml_currencies::Config for Runtime {
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

// orml unknown tokens
impl orml_unknown_tokens::Config for Runtime {
    type Event = Event;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        // System support stuff.
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        ParachainSystem: cumulus_pallet_parachain_system::{
            Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
        } = 1,
        Utility: pallet_utility::{Pallet, Call, Storage, Event} = 2,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 4,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 5,
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 6,

        // Monetary stuff.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,

        // Collator support. The order of these 4 are important and shall not change.
        Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
        CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
        Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
        AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,

        // XCM helpers.
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Storage, Event<T>} = 30,
        PolkadotXcm: pallet_xcm::{Pallet, Storage, Call, Event<T>, Origin, Config} = 31,
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,

        // ORML XCMP
        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 40,
        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>} = 41,
        OrmlXcm: orml_xcm::{Pallet, Call, Event<T>} = 42,
        UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 43,
        Currencies: orml_currencies::{Pallet, Call} = 44,

        // Governance stuff
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 50,
        // Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 51,
        TechnicalCommittee: pallet_collective::<Instance2> = 51,
        TechnicalCommitteeMembership: pallet_membership::<Instance2> = 52,
        Democracy: pallet_democracy = 53,

        // Include the custom pallet in the runtime.
        QuadraticFunding: pallet_qf::{Pallet, Call, Storage, Event<T>} = 60,
        DaoCoreModule: dao_core::{Pallet, Call, Storage, Event<T>} = 61,
        DoraRewards: pallet_dora_rewards::{Pallet, Call, Storage, Event<T>, Config<T>} = 62,

        // Sudo
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 255,
    }
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_session, SessionBench::<Runtime>]
        [pallet_timestamp, Timestamp]
        [pallet_collator_selection, CollatorSelection]
        [cumulus_pallet_xcmp_queue, XcmpQueue]
    );
}

impl_runtime_apis! {
    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info(header)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> (Weight, Weight) {
            log::info!("try-runtime::on_runtime_upgrade parachain-template.");
            let weight = Executive::try_runtime_upgrade().unwrap();
            (weight, RuntimeBlockWeights::get().max_block)
        }

        fn execute_block_no_check(block: Block) -> Weight {
            Executive::execute_block_no_check(block)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{Benchmarking, list_benchmark, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
            list_benchmark!(list, extra, pallet_balances, Balances);
            list_benchmark!(list, extra, pallet_timestamp, Timestamp);
            list_benchmark!(list, extra, pallet_qf, QuadraticFunding);
            list_benchmark!(list, extra, pallet_dora_rewards, DoraRewards);
            list_benchmark!(list, extra, pallet_collator_selection, CollatorSelection);

            let storage_info = AllPalletsWithSystem::storage_info();
            return (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}

            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
            impl cumulus_pallet_session_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_collator_selection, CollatorSelection);
            add_benchmark!(params, batches, pallet_qf, QuadraticFunding);
            add_benchmark!(params, batches, pallet_dora_rewards, DoraRewards);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
    fn check_inherents(
        block: &Block,
        relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
    ) -> sp_inherents::CheckInherentsResult {
        let relay_chain_slot = relay_state_proof
            .read_slot()
            .expect("Could not read the relay chain slot from the proof");

        let inherent_data =
            cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
                relay_chain_slot,
                sp_std::time::Duration::from_secs(6),
            )
            .create_inherent_data()
            .expect("Could not create the timestamp inherent data");

        inherent_data.check_extrinsics(block)
    }
}

cumulus_pallet_parachain_system::register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
    CheckInherents = CheckInherents,
}
