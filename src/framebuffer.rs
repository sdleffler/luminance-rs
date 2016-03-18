use texture;

pub trait HasFramebuffer {
	type AFramebuffer;

	fn new<'a, D>(size: D::Size, mipmaps: u32) -> Result<Self::AFramebuffer, FramebufferError<'a>> where D: texture::Dimensionable;
}

pub enum FramebufferError<'a> {
	Incomplete(&'a str)
}
