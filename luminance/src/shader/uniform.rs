//! Shader uniforms and associated operations.
//!
//! Uniforms kick in several and useful ways. They’re used to customize shaders.

use std::marker::PhantomData;

use linear::*;
use pixel::{self, Pixel};
use texture::{self, Dimensionable, Layerable, HasTexture, Texture};

