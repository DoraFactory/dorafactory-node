//! Test utilities

use crate::{self as pallet_qf, Config, Event as QFEvent};
use cumulus_primitives_core::relay_chain::BlockNumber as RelayChainBlockNumber;
use cumulus_primitives_core::PersistedValidationData;
use cumulus_primitives_parachain_inherent::ParachainInherentData;
// use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::{
    construct_runtime,
    dispatch::UnfilteredDispatchable,
    inherent::{InherentData, ProvideInherent},
    parameter_types,
    traits::{ConstBool, ConstU32, GenesisBuild, Nothing, OnFinalize, OnInitialize},
    PalletId,
};
use frame_system::{EnsureRoot, RawOrigin};
use orml_currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;
use primitives::{AccountId, Amount, BlockNumber, CurrencyId, DOLLARS};
use sp_core::H256;
use sp_io;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
    Perbill,
};
use sp_std::convert::{From, TryInto};
pub type Balance = u128;

pub type ReserveIdentifier = [u8; 8];

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        QuadraticFunding: pallet_qf::{Pallet, Call, Storage, Event<T>},
        Currencies: orml_currencies::{Pallet, Call},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
    }
);

parameter_types! {
    pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type SelfParaId = ParachainId;
    type Event = Event;
    type OnSystemEvent = ();
    type OutboundXcmpMessageSource = ();
    type XcmpMessageHandler = ();
    type ReservedXcmpWeight = ();
    type DmpMessageHandler = ();
    type ReservedDmpWeight = ();
    type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Runtime {
    type BaseCallFilter = Nothing;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Index = u64;
    type Call = Call;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Runtime {
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type MaxLocks = ();
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}

parameter_types! {
    pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account_truncating();
}

impl orml_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = i64;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = ();
    type MaxLocks = ConstU32<100_000>;
    type MaxReserves = ConstU32<100_000>;
    type ReserveIdentifier = ReserveIdentifier;
    type DustRemovalWhitelist = Nothing;
    type OnNewTokenAccount = ();
    type OnKilledTokenAccount = ();
}

pub const NATIVE_CURRENCY_ID: CurrencyId = CurrencyId::DORA;

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = NATIVE_CURRENCY_ID;
}

impl orml_currencies::Config for Runtime {
    type MultiCurrency = Tokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}
// pub type NativeCurrency = NativeCurrencyOf<Runtime>;
pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Runtime, Balances, i64, u64>;

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
    // minimal number of units to reserve to get qualified to vote
    pub const ReserveUnit: u128 = 1000000000000;
    // pub const StringLimit: u32 = 32;
}

// qf pallet config
impl Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MultiCurrency = Currencies;
    type PalletId = QuadraticFundingPalletId;
    // Origin who can control the round
    type AdminOrigin = EnsureRoot<u64>;
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
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();
    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(1, 100 * DOLLARS), (2, 100 * DOLLARS), (3, 100 * DOLLARS)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
