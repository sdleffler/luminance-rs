use blending::{Equation, Factor};
use depth_test::DepthTest;
use face_culling::FaceCulling;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
  pub(crate) blending: Option<(Equation, Factor, Factor)>,
  pub(crate) depth_test: DepthTest,
  pub(crate) face_culling: Option<FaceCulling>
}

impl RenderState {
  pub fn set_blending<B>(self, blending: B) -> Self where B: Into<Option<(Equation, Factor, Factor)>> {
    RenderState {
      blending: blending.into(),
      .. self
    }
  }

  pub fn blending(&self) -> Option<(Equation, Factor, Factor)> {
    self.blending
  }

  pub fn set_depth_test(self, depth_test: DepthTest) -> Self {
    RenderState {
      depth_test,
      .. self
    }
  }

  pub fn depth_test(&self) -> DepthTest {
    self.depth_test
  }

  pub fn set_face_culling<FC>(self, face_culling: FC) -> Self where FC: Into<Option<FaceCulling>> {
    RenderState {
      face_culling: face_culling.into(),
      .. self
    }
  }

  pub fn face_culling(&self) -> Option<FaceCulling> {
    self.face_culling
  }
}

impl Default for RenderState {
  fn default() -> Self {
    RenderState {
      blending: None,
      depth_test: DepthTest::Enabled,
      face_culling: Some(FaceCulling::default())
    }
  }
}
