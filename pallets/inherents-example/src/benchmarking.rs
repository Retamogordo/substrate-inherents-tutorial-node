//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set() {
		let value: T::InherentDataType = Default::default();
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		set(RawOrigin::Signed(caller), value);

		assert_eq!(InherentDataType::<T>::get(), Some(value));
	}

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
