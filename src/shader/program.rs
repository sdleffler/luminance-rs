use shader::stage::HasStage;
use shader::uniform::{HasUniform, UniformName};

pub trait HasProgram: HasStage + HasUniform {
  type Program;

  ///
  fn new(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage) -> Self::Program;
  ///
  fn map_uniform(program: &Self::Program, name: UniformName) -> Option<Self::U>;
}
