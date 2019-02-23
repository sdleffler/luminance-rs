use luminance::vertex::VertexAttribSemantics;
use luminance_derive::Vertex;

type Position = [f32; 3];
type Normal = [f32; 3];
type Color = [f32; 4];

#[test]
fn derive_simple_semantics() {
  #[derive(Clone, Copy, Debug)]
  enum Semantics {
    Position = 0,
    Normal = 1,
    Color = 2
  }

  impl VertexAttribSemantics for Semantics {
    fn index(&self) -> usize {
      *self as usize
    }
  }

  #[derive(Clone, Copy, Debug, Vertex)]
  struct Vertex {
    #[vertex(semantics = "Semantics::Position")]
    pos: Position,
    #[vertex(semantics = "Semantics::Normal")]
    nor: Normal,
    #[vertex(semantics = "Semantics::Color")]
    col: Color
  }
}
