/// Convenient inference helper to export the interface without having to use explicit types.

use buffer::{Buffer, HasBuffer};
use core::marker::PhantomData;

pub struct Device<T>(PhantomData<T>);

impl<C> Device<C> where C: HasBuffer {
  pub fn new_buffer<A, T>(a: A, size: usize) -> Buffer<C, A, T> {
    Buffer::new(a, size)
  }
}
