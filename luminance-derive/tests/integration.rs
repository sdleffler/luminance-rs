use luminance::vertex::VertexAttribSem;
use luminance_derive::{Vertex, VertexAttribSem};

type Position = [f32; 3];
type Normal = [f32; 3];
type Color = [f32; 4];

#[test]
fn derive_simple_semantics() {
  #[derive(Clone, Copy, Debug, VertexAttribSem)]
  enum Semantics {
    #[sem(name = "position")]
    Position = 0,
    #[sem(name = "normal")]
    Normal = 1,
    #[sem(name = "color")]
    Color = 2
  }

  #[derive(Clone, Copy, Debug, Vertex)]
  struct Vertex {
    #[vertex(sem = "Semantics::Position")]
    pos: Position,
    #[vertex(sem = "Semantics::Normal")]
    nor: Normal,
    #[vertex(sem = "Semantics::Color")]
    col: Color
  }
}
