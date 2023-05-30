#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{Decode, Encode};
use sp_inherents::{InherentIdentifier, IsFatalError};
use sp_runtime::traits::Zero;
use sp_std::{borrow::ToOwned, vec::Vec};
//use sp_std::fmt::*;

/// A pallet demontrating an example of usage of inherent extrinsics.
/// Docs on inherent extrinsics are available here:
/// https://paritytech.github.io/substrate/master/sp_inherents/index.html
pub use pallet::*;
/// Type for Scale-encoded data provided by the block author
type InherentRawType = Vec<u8>;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"weat_orc";

pub trait DeconstructableAsFloat {
	fn deconstruct(&self) -> f32;
}

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
			+ MaxEncodedLen
			+ DeconstructableAsFloat;
	}

	// Storage items for inherent data created by the block author
	#[pallet::storage]
	pub type StoredInherentData<T: Config> = StorageValue<_, T::InherentDataType, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn weather_order)]
	pub type WeatherOrder<T: Config> = StorageValue<_, (i16, i16), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Triggered when 'weather_data_set' transaction succedes
		WeatherDataSet { data: <str as ToOwned>::Owned },
		/// Triggered when a user submits geo location for weather report they are interested to
		/// receive from the oracle
		WeatherOrderSet { latlong: (<str as ToOwned>::Owned, <str as ToOwned>::Owned) },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Triggered when the inherent data is already set for the current block
		AlreadySet,
		/// Triggered when the weather order is already set for the current block
		WeatherOrderAlreadySet,
		/// Latitude/Longitude format is similar to signed float number
		FailedParsingLatitudeOrLongitude,
		UnableToCreateEventData,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Unsigned extrinsic submitted by create_inherent(..)
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set())]
		pub fn weather_data_set(
			origin: OriginFor<T>,
			weather_data: T::InherentDataType,
		) -> DispatchResult {
			// as this call is created by block auth it is supposed to be unsigned
			ensure_none(origin)?;
			ensure!(StoredInherentData::<T>::get().is_none(), Error::<T>::AlreadySet);

			StoredInherentData::<T>::put(&weather_data);

			let mut s = <str as ToOwned>::Owned::new();
			sp_std::fmt::write(&mut s, format_args!("{} °C", weather_data.deconstruct()))
				.map_err(|_| Error::<T>::UnableToCreateEventData)?;

			Self::deposit_event(Event::WeatherDataSet { data: s });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::order_weather_data())]
		pub fn order_weather_data(
			origin: OriginFor<T>,
			lat_str: <str as ToOwned>::Owned,
			long_str: <str as ToOwned>::Owned,
		) -> DispatchResult {
			ensure_signed(origin)?;

			let lat = lat_str
				.parse::<f32>()
				.map_err(|_| Error::<T>::FailedParsingLatitudeOrLongitude)?;
			let long = long_str
				.parse::<f32>()
				.map_err(|_| Error::<T>::FailedParsingLatitudeOrLongitude)?;

			let lat = (lat * 10.0) as i16;
			let long = (long * 10.0) as i16;

			ensure!(WeatherOrder::<T>::get().is_none(), Error::<T>::WeatherOrderAlreadySet);

			WeatherOrder::<T>::put(&(lat, long));

			Self::deposit_event(Event::WeatherOrderSet { latlong: (lat_str, long_str) });

			Ok(())
		}
	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			// remove the inherent from storage upon block initialization
			StoredInherentData::<T>::kill();
			WeatherOrder::<T>::kill();

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
			Self::get_and_decode_data(data)
				.map(|weather_data| Call::weather_data_set { weather_data })
		}
		// Determine if a call is an inherent extrinsic
		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::weather_data_set { .. })
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_and_decode_data(data: &InherentData) -> Option<T::InherentDataType> {
			let res = data
				.get_data::<InherentRawType>(&INHERENT_IDENTIFIER)
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
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		true
	}
}
