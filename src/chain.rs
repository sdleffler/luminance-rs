pub struct Chain<A, B> {
  pub left: A,
  pub right: B
}

macro_rules! chain {
  ($t:ty) => {( $t )};
  
  ($a:ty, $($r:tt)*) => {( Chain<$a, chain![$($r)*]> )}
}
