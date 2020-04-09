//! WebGL 2.0 backend support.

//mod state;

// Get all the generated symbols for WebGL.
pub(crate) mod webgl {
  #![allow(missing_docs, unused_parens, non_camel_case_types, warnings)]
  include!(concat!(env!("OUT_DIR"), "/webgl_stdweb.rs"));
}

pub(crate) mod state;
