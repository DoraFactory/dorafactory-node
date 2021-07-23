#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
use frame_support::{
	PalletId,
	traits::{Currency, ReservableCurrency, OnUnbalanced, Get, UnfilteredDispatchable},
	codec::{Encode, Decode},
	weights::GetDispatchInfo
};
use sp_std::{vec, vec::Vec, convert::{TryInto}};
use sp_std::boxed::Box;
use sp_runtime::traits::{Hash, AccountIdConversion};


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Orgnization<AccountId> {
	pub org_type: u32,
	pub description: Vec<u8>,
	pub owner: AccountId,
	pub members: Vec<AccountId>,
}

type OrgnizationOf<T> = Orgnization<<T as frame_system::Config>::AccountId>;

#[frame_support::pallet]
pub mod pallet {
	pub use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	pub use frame_system::pallet_prelude::*;
	pub use super::*;


	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: Parameter + UnfilteredDispatchable<Origin = Self::Origin> + GetDispatchInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn next_org_id)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub(super) type NextOrgId<T> = StorageValue<_, u32, ValueQuery>; 

	#[pallet::storage]
	#[pallet::getter(fn orgnizations)]
	pub(super) type Orgnizations<T: Config> = StorageMap<_, Blake2_128Concat, u32, OrgnizationOf<T>, ValueQuery>;


	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", T::Hash = "Hash")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [ord_id, owner]
		OrgRegistered(u32, T::AccountId),
		OrgJoined(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		OrgnizationNotExist,
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
		pub fn create(origin: OriginFor<T>, description:Vec<u8> ) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;
			let member = super::Orgnization {
				org_type: 1,
				description: description.clone(),
				owner: who.clone(),
				members: [who.clone()].to_vec(),
			};
			let org_id = <NextOrgId<T>>::get().checked_add(1).unwrap();
			Orgnizations::<T>::insert(org_id, member);
			<NextOrgId<T>>::put(org_id);
			Self::deposit_event(Event::OrgRegistered(org_id, who));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn join(origin: OriginFor<T>, org_id:u32 ) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;
			ensure!(Orgnizations::<T>::contains_key(&org_id), Error::<T>::OrgnizationNotExist);
			Orgnizations::<T>::mutate(org_id, |org| {
				org.members.push(who.clone());
			});
			Self::deposit_event(Event::OrgJoined(org_id, who));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn invoke(origin: OriginFor<T>, ord_id: u32, pallet: Box<<T as Config>::Call>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			ensure!(Self::validate_member(who.clone(), ord_id), Error::<T>::NotValidOrgMember);
			let _ = pallet.dispatch_bypass_filter(origin);
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
	pub fn validate_member(account_id: T::AccountId, ord_id: u32) -> bool {
		if !Orgnizations::<T>::contains_key(ord_id) {
			false
		} else {
			let members = Orgnizations::<T>::get(ord_id).members;
			match members.binary_search(&account_id) {
				Ok(_) => {
					true
				}
				Err(_) => {
					false
				}
			}
		}
	}
}
