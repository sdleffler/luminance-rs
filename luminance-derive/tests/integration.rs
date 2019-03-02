use luminance::vertex::VertexAttribSem;
use luminance_derive::{Vertex, VertexAttribSem};

#[test]
fn derive_simple_semantics() {
  #[derive(Clone, Copy, Debug, Eq, PartialEq, VertexAttribSem)]
  enum Semantics {
    #[sem(name = "position", repr = "[f32; 3]", type_name = "VertexPosition")]
    Position,
    #[sem(name = "normal", repr = "[f32; 3]", type_name = "VertexNormal")]
    Normal,
    #[sem(name = "color", repr = "[f32; 4]", type_name = "VertexColor")]
    Color
  }

  assert_eq!(Semantics::Position.index(), 0);
  assert_eq!(Semantics::Normal.index(), 1);
  assert_eq!(Semantics::Color.index(), 2);
  assert_eq!(<Semantics as VertexAttribSem>::parse("position"), Some(Semantics::Position));
  assert_eq!(<Semantics as VertexAttribSem>::parse("normal"), Some(Semantics::Normal));
  assert_eq!(<Semantics as VertexAttribSem>::parse("color"), Some(Semantics::Color));
  assert_eq!(<Semantics as VertexAttribSem>::parse("bidule"), None);
  assert_eq!(VertexPosition::new([1., 2., 3.]).repr, [1., 2., 3.]);

  #[derive(Clone, Copy, Debug, Vertex)]
  #[vertex(sem = "Semantics")]
  struct Vertex {
    #[vertex(sem = "Semantics::Position")]
    pos: VertexPosition,
    #[vertex(sem = "Semantics::Normal")]
    nor: VertexNormal,
    #[vertex(sem = "Semantics::Color")]
    col: VertexColor
  }
}
