#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

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

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, match_type, parameter_types,
    traits::{
        Contains, EnsureOneOf, EqualPrivilegeOnly, Everything, KeyOwnerProofSystem, Nothing,
        Randomness, StorageInfo,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
        IdentityFee, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    },
    PalletId,
};
use frame_system::EnsureRoot;

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot Imports
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_runtime_common::RocksDbWeight;

// XCM Imports
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, EnsureXcmOrigin,
    FixedRateOfFungible, FixedWeightBounds, LocationInverter, ParentIsPreset, RelayChainAsNative,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::XcmExecutor;

// ORML Imports
use orml_currencies::BasicCurrencyAdapter;
use orml_traits::{location::AbsoluteReserveProvider, parameter_type_with_key};
use orml_xcm_support::{
    DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset,
};

/// local Imports
/// Import the qudratic-funding pallet.
pub use pallet_qf;

/// import the dora-rewards pallet
pub use pallet_dora_rewards;

pub use primitives::{
    AccountId, Address, Amount, Balance, BlockNumber, CurrencyId, Hash, Index, Signature, CENTS,
    DAYS, DOLLARS, EXISTENTIAL_DEPOSIT, HOURS, MICROUNIT, MILLICENTS, MILLISECS_PER_BLOCK,
    MILLIUNIT, MINUTES, SLOT_DURATION, UNIT,
};

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
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
    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    use sp_runtime::{generic, traits::BlakeTwo256};

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

// To learn more about runtime versioning and what each of the following value means:
//   https://substrate.dev/docs/en/knowledgebase/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("DORA KSM Parachain"),
    impl_name: create_runtime_str!("DORA KSM Parachain"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 0,
};

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
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 128;
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
    pub const TreasuryPalletId: PalletId = PalletId(*b"dora/try");
}

// Configure FRAME pallets to include in runtime.
impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = frame_support::traits::Everything;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
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
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 128 is the dora ksm prefix.
    type SS58Prefix = SS58Prefix;
    /// The action to take on a Runtime Upgrade
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

parameter_types! {
    pub const Period: u32 = 6 * HOURS;
    pub const Offset: u32 = 0;
    // 自定义
    pub const MaxAuthorities: u32 = 100_000;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
    }
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

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
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
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
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
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * MICROUNIT;
    pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
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
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    pub const RelayNetwork: NetworkId = NetworkId::Any;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the default `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, Origin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<Origin>,
);

parameter_types! {
    // One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
    pub UnitWeightCost: Weight = 1_000_000_000;
    pub const MaxInstructions: u32 = 100;
}

match_type! {
    pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Here } |
        MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
    };
}

match_type! {
    pub type SpecParachain: impl Contains<MultiLocation> = {
        MultiLocation {parents: 1, interior: X1(Parachain(2115))}
    };
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
    // ^^^ Parent and its exec plurality get free execution
    AllowUnpaidExecutionFrom<SpecParachain>,
);

pub type LocalAssetTransactor = MultiCurrencyAdapter<
    Currencies,
    UnknownTokens,
    IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
    AccountId,
    LocationToAccountId,
    CurrencyId,
    CurrencyIdConvert,
    DepositToAlternative<NativeTreasuryAccount, Currencies, CurrencyId, AccountId, Balance>,
>;

// TODO: add treasury;
// pub struct ToTreasury;
// impl TakeRevenue for ToTreasury {
//     fn take_revenue(revenue: MultiAsset) {
//         if let MultiAsset {
//             id: Concrete(location),
//             fun: Fungible(amount),
//         } = revenue
//         {
//             if let Some(currency_id) = CurrencyIdConvert::convert(location) {
//                 // Ensure AcalaTreasuryAccount have ed requirement for native asset, but don't need
//                 // ed requirement for cross-chain asset because it's one of whitelist accounts.
//                 // Ignore the result.
//                 let _ = Currencies::deposit(currency_id, &NativeTreasuryAccount::get(), amount);
//             }
//         }
//     }
// }

/// Trader - The means of purchasing weight credit for XCM execution.
/// We need to ensure we have at least one rule per token we want to handle or else
/// the xcm executor won't know how to charge fees for a transfer of said token.
pub type Trader = (
    FixedRateOfFungible<KsmPerSecond, ()>,
    FixedRateOfFungible<NativePerSecond, ()>,
    FixedRateOfFungible<NativeNewPerSecond, ()>,
);

parameter_types! {
    pub KsmPerSecond: (AssetId, u128) = (MultiLocation::parent().into(), ksm_per_second());
    pub NativePerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(2115), GeneralKey(b"DORA".to_vec()))
        ).into(),
        // DORA:KSM = 50:1
        ksm_per_second() * 50
    );
    pub NativeNewPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            0,
            X1(GeneralKey(b"DORA".to_vec()))
        ).into(),
        // DORA:KSM = 50:1
        ksm_per_second() * 50
    );
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type Call = Call;
    type XcmSender = XcmRouter;
    // How to withdraw and deposit an asset.
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
    type IsTeleporter = (); // Teleporting is disabled.
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    // type Trader = UsingComponents<IdentityFee<Balance>, RelayLocation, AccountId, Balances, ()>;
    type Trader = Trader;
    type ResponseHandler = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
}

parameter_types! {
    pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Nothing;
    // ^ Disable dispatchable execute on the XCM pallet.
    // Needs to be `Everything` for local testing.
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type Origin = Origin;
    type Call = Call;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    // ^ Override for AdvertisedXcmVersion default
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

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
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = 1 * DOLLARS;
    pub const PreimageByteDeposit: Balance = 1 * CENTS;
}

impl pallet_preimage::Config for Runtime {
    type Event = Event;
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type MaxSize = PreimageMaxSize;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

// Configure the pallet-qf in pallets/quadratic-funding.
parameter_types! {
    // pow(10,12) => Unit, for easy fee control, we use pow(10,9)
    pub const VoteUnit: u128 = 1000000000;
    // The base of unit per vote, should be 1 Unit of token for each vote
    pub const NumberOfUnit: u128 = 1000;
    // The ratio of fee for each trans, final value should be FeeRatio/NumberOfUnit
    pub const FeeRatio: u128 = 60;
    pub const QuadraticFundingPalletId: PalletId = PalletId(*b"py/quafd");
    pub const NameMinLength: u32 = 3;
    pub const NameMaxLength: u32 = 32;
    pub const AppId: u8 = 1;
    // pub const StringLimit: u32 = 32;
}

/// Configure the pallet-qf in pallets/quadratic-funding.
impl pallet_qf::Config for Runtime {
    type Event = Event;

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
    pub const MaxContributorsNumber: u32 = 5;
}

impl pallet_dora_rewards::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type VestingBlockNumber = cumulus_primitives_core::relay_chain::BlockNumber;
    type VestingBlockProvider =
        cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
    type FirstVestPercentage = FirstVestPercentage;
    type MaxContributorsNumber = MaxContributorsNumber;
}

pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: CurrencyId) -> Option<MultiLocation> {
        match id {
            CurrencyId::KSM => Some(Parent.into()),
            CurrencyId::DORA => Some((Parent, Parachain(2115), GeneralKey("DORA".into())).into()),
        }
    }
}

impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(l: MultiLocation) -> Option<CurrencyId> {
        let dora: Vec<u8> = "DORA".into();
        if l == MultiLocation::parent() {
            return Some(CurrencyId::KSM);
        }

        match l {
            MultiLocation { parents, interior } if parents == 1 => match interior {
                X2(Parachain(2115), GeneralKey(k)) if k == dora => Some(CurrencyId::DORA),
                _ => None,
            },
            MultiLocation { parents, interior } if parents == 0 => match interior {
                X1(GeneralKey(k)) if k == dora => Some(CurrencyId::DORA),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(asset: MultiAsset) -> Option<CurrencyId> {
        if let MultiAsset {
            id: Concrete(id), ..
        } = asset
        {
            Self::convert(id)
        } else {
            Option::None
        }
    }
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account: AccountId) -> MultiLocation {
        X1(AccountId32 {
            network: NetworkId::Any,
            id: account.into(),
        })
        .into()
    }
}

parameter_types! {
    pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
    pub const BaseXcmWeight: Weight = 100_000_000;
    pub const MaxAssetsForTransfer: usize = 2;
}

parameter_type_with_key! {
    pub ParachainMinFee: |location: MultiLocation| -> u128 {
        #[allow(clippy::match_ref_pats)] // false positive
        match (location.parents, location.first_interior()) {
            (1, Some(Parachain(2115))) => 4_000_000_000,
            _ => u128::MAX,
        }
    };
}

impl orml_xtokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type SelfLocation = SelfLocation;
    type MinXcmFee = ParachainMinFee;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type MultiLocationsFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type LocationInverter = LocationInverter<Ancestry>;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type ReserveProvider = AbsoluteReserveProvider;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        // every currency has a zero existential deposit
        match currency_id {
            _ => 0,
        }
    };
}

parameter_types! {
    pub ORMLMaxLocks: u32 = 2;
    pub NativeTreasuryAccount: AccountId = TreasuryPalletId::get().into_account();
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
    type MaxLocks = ORMLMaxLocks;
    type DustRemovalWhitelist = Nothing;
}

impl orml_xcm::Config for Runtime {
    type Event = Event;
    type SovereignOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::DORA;
}

impl orml_currencies::Config for Runtime {
    type Event = Event;
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
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        ParachainSystem: cumulus_pallet_parachain_system::{
            Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
        },
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        ParachainInfo: parachain_info::{Pallet, Storage, Config},
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} ,
        Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>},

        // Monetary stuff.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage},

        // collator support
        Authorship: pallet_authorship::{Pallet, Call, Storage},
        CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
        Aura: pallet_aura::{Pallet, Config<T>},
        AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config},

        // XCM helpers.
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin, Config},
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},

        // ORML XCMP
        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
        OrmlXcm: orml_xcm::{Pallet, Call, Event<T>},
        UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event},
        Currencies: orml_currencies::{Pallet, Call, Event<T>},


        // Include the custom pallet in the runtime.
        QuadraticFunding: pallet_qf::{Pallet, Call, Storage, Event<T>},
        DaoCoreModule: dao_core::{Pallet, Call, Storage, Event<T>},
        DoraRewards: pallet_dora_rewards::{Pallet, Call, Storage, Event<T>, Config<T>},
    }
);

impl_runtime_apis! {
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

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().to_vec()
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

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
            let mut list = Vec::<BenchmarkList>::new();

            list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
            list_benchmark!(list, extra, pallet_balances, Balances);
            list_benchmark!(list, extra, pallet_timestamp, Timestamp);
            list_benchmark!(list, extra, pallet_qf, QuadraticFunding);
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
            // add_benchmark!(params, batches, pallet_session, Session);
            add_benchmark!(params, batches, pallet_qf, QuadraticFunding);

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
