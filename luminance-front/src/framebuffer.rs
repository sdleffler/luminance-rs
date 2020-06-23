use crate::Backend;

pub type Framebuffer<D, CS, DS> = luminance::framebuffer::Framebuffer<Backend, D, CS, DS>;
pub use luminance::framebuffer::{FramebufferError, IncompleteReason};
