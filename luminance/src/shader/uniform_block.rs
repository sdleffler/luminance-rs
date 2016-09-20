use buffer::HasBuffer;

pub trait HasUniformBlock: HasBuffer {
  /// Uniform block representation.
  type UB;

  fn update_uniform_block(ub: &Self::UB, buffer: &Self::ABuffer, index: u32);
}
