/// Convenient inference helper to export the interface without having to use explicit types.

use core::marker::PhantomData;

pub struct Device<T>(PhantomData<T>);

impl<C> Default for Device<C> {
	fn default() -> Device<C> {
		Device(PhantomData)
	}
}

