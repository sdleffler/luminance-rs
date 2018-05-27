#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FaceCulling {
  pub(crate) order: FaceCullingOrder,
  pub(crate) mode: FaceCullingMode
}

impl FaceCulling {
  pub fn new(order: FaceCullingOrder, mode: FaceCullingMode) -> Self {
    FaceCulling { order, mode }
  }
}

impl Default for FaceCulling {
  fn default() -> Self {
    FaceCulling::new(FaceCullingOrder::CCW, FaceCullingMode::Back)
  }
}

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

