#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{Decode, Encode};
use sp_inherents::{InherentIdentifier, IsFatalError};
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;

/// A pallet demontrating an example of usage of inherent extrinsics.
/// Docs on inherent extrinsics are available here:
/// https://paritytech.github.io/substrate/master/sp_inherents/index.html
pub use pallet::*;
/// Type for Scale-encoded data provided by the block author
type InherentType = Vec<u8>;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"ext_data";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		/// Actual type of inherent data
		type InherentDataType: Default
			+ Encode
			+ Decode
			+ Clone
			+ Parameter
			+ Member
			+ MaxEncodedLen;
	}

	// Storage items for inherent data created by the block author
	#[pallet::storage]
	pub type StoredInherentData<T: Config> = StorageValue<_, T::InherentDataType, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Triggered when 'set' transaction succedes
		InherentDataSet { data: T::InherentDataType },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Triggered when the inherent data is already set for the current block
		AlreadySet,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Unsigned extrinsic submitted by create_inherent(..)
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set())]
		pub fn set(origin: OriginFor<T>, inherent_data: T::InherentDataType) -> DispatchResult {
			// as this call is created by block auth it is supposed to be unsigned
			ensure_none(origin)?;
			ensure!(StoredInherentData::<T>::get().is_none(), Error::<T>::AlreadySet);

			StoredInherentData::<T>::put(&inherent_data);

			Self::deposit_event(Event::InherentDataSet { data: inherent_data });

			Ok(())
		}
	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			// remove the inherent from storage upon block initialization
			StoredInherentData::<T>::kill();

			Zero::zero()
		}
	}

	// This pallet provides an inherent, as such it implements ProvideInherent trait
	// https://paritytech.github.io/substrate/master/frame_support/inherent/trait.ProvideInherent.html
	#[pallet::inherent]
	impl<T: Config> ProvideInherent for Pallet<T> {
		type Call = Call<T>;
		type Error = InherentError;
		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;
		// This method is used to decide whether this inherent is requiered for the block to be
		// accepted
		fn is_inherent_required(data: &InherentData) -> Result<Option<Self::Error>, Self::Error> {
			// we could return Ok(None) to indicate that this inherent is not required.
			// This happens by default if altenative implementation of is_inherent_required is
			// provided Here for demonstration we return Ok(Some(..)) if inherent data is present
			// and successfully decoded, expecting that inherent is required in this case.
			Ok(Self::get_and_decode_data(data)
				.map(|_| InherentError::InherentRequiredForDataPresent))
		}

		fn create_inherent(data: &InherentData) -> Option<Self::Call> {
			// create and return the extrinsic call if the data could be read and decoded
			Self::get_and_decode_data(data).map(|inherent_data| Call::set { inherent_data })
		}
		// Determine if a call is an inherent extrinsic
		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::set { .. })
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_and_decode_data(data: &InherentData) -> Option<T::InherentDataType> {
			let res = data
				.get_data::<InherentType>(&INHERENT_IDENTIFIER)
				.ok()
				.unwrap_or_default()
				.and_then(|encoded_data| T::InherentDataType::decode(&mut &encoded_data[..]).ok());
			res
		}
	}
}

#[derive(Encode)]
pub enum InherentError {
	InherentRequiredForDataPresent,
	NotPresent123,
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		true
	}
}
