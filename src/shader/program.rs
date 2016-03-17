use shader::stage::HasStage;
use shader::uniform;

pub trait HasProgram: HasStage + uniform::HasUniform {
  type Program;

  fn new(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage) -> Self::Program;
}
