# Dora-Rewards
This is a Rewards pallet which distributes DORAs to the contributors who support the DoraFactory's Crowdloan

## Config 
1. Import the dependency in your runtime `Cargo.toml`
```shell
pallet-dora-rewards = {git = "https://github.com/DoraFactory/DoraFactory", default-features = false , branch = "master"}
```

2. Add some Config in your runtime `lib.rs`
```
parameter_types! {
    // this first distribution percentage which depends on you
	pub const FirstVestPercentage: Perbill = Perbill::from_percent(20);
	// max contributor limit which depends on you
	pub const MaxContributorsNumber: u32 = 3;
}

impl pallet_dora_rewards::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type VestingBlockNumber = cumulus_primitives_core::relay_chain::BlockNumber;
	type VestingBlockProvider = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
	type FirstVestPercentage = FirstVestPercentage;
	
}
```

3. pre-fund some asset of this pallet_account which will distribute reward to contributors in your parachain's `chainspec.rs`
```
dora_rewards: parachain_template_runtime::DoraRewardsConfig {
            // this amount depend on you
			funded_amount: 1 << 60,
		},
```

## Usage
> Attention: the first two steps need `Sudo` to operate !
- first, you need to call `initialize_contributors_list` function to set the contributor's contribution info which decide how much DORA can get.
- Second, you need to set ending lease block by `complete_initialization` function htat we can compute the linear reward.

After that contributors can claim their rewards(first reward and linear reward) by `claim_rewards` function
