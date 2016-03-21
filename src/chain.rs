#![feature(type_macros)]

extern crate core;

use core::marker::PhantomData;

struct Chain<A, B> {
  _a: PhantomData<A>,
  _b: PhantomData<B>
}

macro_rules! chain {
  ($t:ty) => {( $t )};
  
  ($a:ty, $($r:tt)*) => {( Chain<$a, chain![$($r)*]> )}
}

type Foo = tuple![bool];

fn main() {
  let foo: Foo = ();
  
  println!("{:?}", foo);
}
