use crate::{Module, Trait};
use frame_support::{
    impl_outer_origin, parameter_types,
    traits::{OnFinalize, OnInitialize},
    weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
pub type Balance = u128;

impl_outer_origin! {
    pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl frame_system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type AccountData = pallet_balances::AccountData<Balance>;
}

type Randomness = pallet_randomness_collective_flip::Module<Test>;
pub type System = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;

parameter_types! {
    pub const ExistentialDeposit: u128 = 100;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Trait for Test {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = ();
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl Trait for Test {
    type Event = ();
    type Randomness = Randomness;
    type KittyIndex = u32;
    type Currency = Balances;
}

pub type Kitties = Module<Test>;

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Kitties::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Kitties::on_initialize(System::block_number());
    }
}
// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let _ = pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 2000), (2, 2000), (3, 2000), (4, 2000), (5, 400)],
    }
    .assimilate_storage(&mut storage);

    let ext = sp_io::TestExternalities::from(storage);
    ext
}
