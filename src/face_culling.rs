use gl;
use gl::types::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum FaceCullingState {
  Enabled,
  Disabled
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceCullingOrder {
  CW,
  CCW
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceCullingMode {
  Front,
  Back,
  Both
}
