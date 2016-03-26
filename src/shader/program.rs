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
  ///
  fn update_uniforms<F>(program: &Self::Program, f: F) where F: Fn();
}

#[derive(Debug)]
pub struct Program<C, U> where C: HasProgram {
	pub repr: C::Program,
  pub uniforms: U
}

impl<C, U> Drop for Program<C, U> where C: HasProgram {
  fn drop(&mut self) {
    C::free_program(&mut self.repr)
  }
}

impl<C, U> Program<C, U> where C: HasProgram {
	pub fn new<F>(tess: Option<(&Stage<C, TessellationControlShader>, &Stage<C, TessellationEvaluationShader>)>, vertex: &Stage<C, VertexShader>, geometry: Option<&Stage<C, GeometryShader>>, fragment: &Stage<C, FragmentShader>, f: F) -> Result<Program<C, U>, ProgramError> where F: Fn() -> U {
		C::new_program(tess.map(|(tcs, tes)| (&tcs.repr, &tes.repr)), &vertex.repr, geometry.map(|g| &g.repr), &fragment.repr).map(|repr| Program {
      repr: repr,
      uniforms: f()
    })
	}

  pub fn uniform<T>(&self, name: &str) -> Option<Uniform<C, T>> where T: Uniformable {
    C::map_uniform(&self.repr, UniformName::StringName(String::from(name))).map(|u| Uniform::new(u))
  }

  pub fn update<F>(&self, f: F) where F: Fn() {
    C::update_uniforms(&self.repr, f)
  }
}

#[derive(Debug)]
pub enum ProgramError {
  LinkFailed(String)
}
