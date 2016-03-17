use shader::stage::HasStage;

pub trait HasProgram: HasStage {
  type Program;

}
