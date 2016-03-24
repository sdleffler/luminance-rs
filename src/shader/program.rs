use shader::stage::*;
use shader::uniform::{HasUniform, UniformName};

pub trait HasProgram: HasStage + HasUniform {
  type Program;

  ///
  fn new_program(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage) -> Self::Program;
  ///
  fn map_uniform(program: &Self::Program, name: UniformName) -> Option<Self::U>;
}

pub struct Program<C> where C: HasProgram {
	pub repr: C::Program
}

impl<C> Program<C> where C: HasProgram {
	pub fn new(tess: Option<(&Stage<C, TessellationControlShader>, &Stage<C, TessellationEvaluationShader>)>, vertex: &Stage<C, VertexShader>, geometry: Option<&Stage<C, GeometryShader>>, fragment: &Stage<C, FragmentShader>) -> C::Program {
		C::new_program(tess.map(|(tcs, tes)| (&tcs.repr, &tes.repr)), &vertex.repr, geometry.map(|g| &g.repr), &fragment.repr)
	}
}
