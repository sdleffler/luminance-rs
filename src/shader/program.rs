use shader::stage::*;
use shader::uniform::{HasUniform, Uniform, Uniformable, UniformName};

pub trait HasProgram: HasStage + HasUniform {
  type Program;

  ///
  fn new_program(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage) -> Result<Self::Program, ProgramError>;
  ///
  fn free_program(program: &mut Self::Program);
  ///
  fn map_uniform(program: &Self::Program, name: UniformName) -> Option<Self::U>;
}

#[derive(Debug)]
pub struct Program<C> where C: HasProgram {
	pub repr: C::Program
}

impl<C> Drop for Program<C> where C: HasProgram {
  fn drop(&mut self) {
    C::free_program(&mut self.repr)
  }
}

impl<C> Program<C> where C: HasProgram {
	pub fn new(tess: Option<(&Stage<C, TessellationControlShader>, &Stage<C, TessellationEvaluationShader>)>, vertex: &Stage<C, VertexShader>, geometry: Option<&Stage<C, GeometryShader>>, fragment: &Stage<C, FragmentShader>) -> Result<Program<C>, ProgramError> {
		C::new_program(tess.map(|(tcs, tes)| (&tcs.repr, &tes.repr)), &vertex.repr, geometry.map(|g| &g.repr), &fragment.repr).map(|repr| Program { repr: repr })
	}

  pub fn uniform<T>(&self, name: &str) -> Option<Uniform<C, T>> where T: Uniformable {
    C::map_uniform(&self.repr, UniformName::StringName(String::from(name))).map(|u| Uniform::new(u))
  }
}

#[derive(Debug)]
pub enum ProgramError {
  LinkFailed(String)
}
