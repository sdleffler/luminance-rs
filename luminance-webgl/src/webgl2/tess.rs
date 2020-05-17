//! WebGL2 tessellation implementation.

use luminance::backend::buffer::Buffer as _;
use luminance::backend::tess::{
  IndexSlice as IndexSliceBackend, InstanceSlice as InstanceSliceBackend, Tess as TessBackend,
  VertexSlice as VertexSliceBackend,
};
use luminance::tess::{Interleaved, TessError, TessIndex, TessVertexData};
use luminance::vertex::{Vertex, VertexBufferDesc};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::WebGlVertexArrayObject;

use crate::webgl2::buffer::Buffer;
use crate::webgl2::state::WebGL2State;
use crate::webgl2::WebGL2;

/// All the extra data required when doing indexed drawing.
struct IndexedDrawState<I>
where
  I: TessIndex,
{
  buffer: Buffer<I>,
  restart_index: Option<I>,
}

struct TessRaw<I>
where
  I: TessIndex,
{
  vao: WebGlVertexArrayObject,
  mode: u32,
  vert_nb: usize,
  inst_nb: usize,
  patch_vert_nb: usize,
  index_state: Option<IndexedDrawState<I>>,
  state: Rc<RefCell<WebGL2State>>,
}

impl<I> Drop for TessRaw<I>
where
  I: TessIndex,
{
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();
    state.unbind_vertex_array(&self.vao);
    state.ctx.delete_vertex_array(Some(&self.vao));
  }
}

pub struct InterleavedTess<V, I, W>
where
  V: Vertex,
  I: TessIndex,
  W: Vertex,
{
  raw: TessRaw<I>,
  vertex_buffer: Option<Buffer<V>>,
  instance_buffer: Option<Buffer<W>>,
}

unsafe impl<V, I, W> TessBackend<V, I, W, Interleaved> for WebGL2
where
  V: TessVertexData<Interleaved, Data = Vec<V>>,
  I: TessIndex,
  W: TessVertexData<Interleaved, Data = Vec<W>>,
{
  type TessRepr = InterleavedTess<V, I, W>;

  unsafe fn build(
    &mut self,
    vertex_data: Option<V::Data>,
    index_data: Vec<I>,
    instance_data: Option<W::Data>,
    mode: Mode,
    vert_nb: usize,
    inst_nb: usize,
    restart_index: Option<I>,
  ) -> Result<Self::TessRepr, TessError> {
    let patch_vert_nb = match mode {
      Mode::Patch(nb) => nb,
      _ => 0,
    };

    let vao = self
      .state
      .borrow_mut()
      .create_vertex_array()
      .ok_or_else(|| TessError::cannot_create("the backend failed to create the VAO"))?;

    // force binding the vertex array so that previously bound vertex arrays (possibly the same
    // handle) don’t prevent us from binding here
    self
      .state
      .borrow_mut()
      .bind_vertex_array(Some(&vao), Bind::Forced);

    let vertex_buffer = build_interleaved_vertex_buffer(self, vertex_data)?;

    // in case of indexed render, create an index buffer
    let index_state = build_index_buffer(self, index_data, restart_index)?;

    let instance_buffer = build_interleaved_vertex_buffer(self, instance_data)?;

    let mode = opengl_mode(mode);
    let state = self.state.clone();
    let raw = TessRaw {
      vao,
      mode,
      vert_nb,
      inst_nb,
      patch_vert_nb,
      index_state,
      state,
    };

    Ok(InterleavedTess {
      raw,
      vertex_buffer,
      instance_buffer,
    })
  }
}

fn build_interleaved_vertex_buffer<V>(
  webgl2: &mut WebGL2,
  vertices: Option<Vec<V>>,
) -> Result<Option<Buffer<V>>, TessError>
where
  V: Vertex,
{
  match vertices {
    Some(vertices) => {
      let fmt = V::vertex_desc();

      let vb = if vertices.is_empty() {
        None
      } else {
        let vb = unsafe { webgl2.from_vec(vertices)? };

        // force binding as it’s meaningful when a vao is bound
        unsafe {
          webgl2
            .state
            .borrow_mut()
            .bind_array_buffer(Some(vb.handle()), Bind::Forced)
        };
        set_vertex_pointers(&mut webgl2.state.borrow_mut().ctx, &fmt);

        Some(vb)
      };

      Ok(vb)
    }

    None => Ok(None),
  }
}

/// Give WebGL types information on the content of the VBO by setting vertex descriptors and
/// pointers to buffer memory.
fn set_vertex_pointers(ctx: &mut WebGl2RenderingContext, descriptors: &[VertexBufferDesc]) {
  // this function sets the vertex attribute pointer for the input list by computing:
  //   - The vertex attribute ID: this is the “rank” of the attribute in the input list (order
  //     matters, for short).
  //   - The stride: this is easily computed, since it’s the size (bytes) of a single vertex.
  //   - The offsets: each attribute has a given offset in the buffer. This is computed by
  //     accumulating the size of all previously set attributes.
  let offsets = aligned_offsets(descriptors);
  let vertex_weight = offset_based_vertex_weight(descriptors, &offsets);

  for (desc, off) in descriptors.iter().zip(offsets) {
    set_component_format(ctx, vertex_weight, off, desc);
  }
}

/// Compute offsets for all the vertex components according to the alignments provided.
fn aligned_offsets(descriptor: &[VertexBufferDesc]) -> Vec<usize> {
  let mut offsets = Vec::with_capacity(descriptor.len());
  let mut off = 0;

  // compute offsets
  for desc in descriptor {
    let desc = &desc.attrib_desc;
    off = off_align(off, desc.align); // keep the current component descriptor aligned
    offsets.push(off);
    off += component_weight(desc); // increment the offset by the pratical size of the component
  }

  offsets
}

/// Align an offset.
#[inline]
fn off_align(off: usize, align: usize) -> usize {
  let a = align - 1;
  (off + a) & !a
}

/// Weight in bytes of a vertex component.
fn component_weight(f: &VertexAttribDesc) -> usize {
  dim_as_size(f.dim) as usize * f.unit_size
}

fn dim_as_size(d: VertexAttribDim) -> usize {
  match d {
    VertexAttribDim::Dim1 => 1,
    VertexAttribDim::Dim2 => 2,
    VertexAttribDim::Dim3 => 3,
    VertexAttribDim::Dim4 => 4,
  }
}

/// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
/// aligned.
fn offset_based_vertex_weight(descriptors: &[VertexBufferDesc], offsets: &[usize]) -> usize {
  if descriptors.is_empty() || offsets.is_empty() {
    return 0;
  }

  off_align(
    offsets[offsets.len() - 1] + component_weight(&descriptors[descriptors.len() - 1].attrib_desc),
    descriptors[0].attrib_desc.align,
  )
}

/// Set the vertex component OpenGL pointers regarding the index of the component and the vertex
/// stride.
fn set_component_format(
  ctx: &mut WebGl2RenderingContext,
  stride: usize,
  off: usize,
  desc: &VertexBufferDesc,
) {
  let attrib_desc = &desc.attrib_desc;
  let index = desc.index as u32;

  unsafe {
    match attrib_desc.ty {
      VertexAttribType::Floating => {
        ctx.vertex_attrib_pointer_with_i32(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          false,
          stride as _,
          off as _,
        );
      }

      VertexAttribType::Integral(Normalized::No)
      | VertexAttribType::Unsigned(Normalized::No)
      | VertexAttribType::Boolean => {
        // non-normalized integrals / booleans
        ctx.vertex_attrib_i_pointer_with_i32(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          stride as _,
          off as _,
        );
      }

      _ => {
        // normalized integrals
        ctx.vertex_attrib_pointer_with_i32(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          true,
          stride as _,
          off as _,
        );
      }
    }

    // set vertex attribute divisor based on the vertex instancing configuration
    let divisor = match desc.instancing {
      VertexInstancing::On => 1,
      VertexInstancing::Off => 0,
    };
    ctx.vertex_attrib_divisor(index, divisor);

    ctx.enable_vertex_attrib_array(index);
  }
}

fn opengl_sized_type(f: &VertexAttribDesc) -> u32 {
  match (f.ty, f.unit_size) {
    (VertexAttribType::Integral(_), 1) => WebGl2RenderingContext::BYTE,
    (VertexAttribType::Integral(_), 2) => WebGl2RenderingContext::SHORT,
    (VertexAttribType::Integral(_), 4) => WebGl2RenderingContext::INT,
    (VertexAttribType::Unsigned(_), 1) | (VertexAttribType::Boolean, 1) => {
      WebGl2RenderingContext::UNSIGNED_BYTE
    }
    (VertexAttribType::Unsigned(_), 2) => WebGl2RenderingContext::UNSIGNED_SHORT,
    (VertexAttribType::Unsigned(_), 4) => WebGl2RenderingContext::UNSIGNED_INT,
    (VertexAttribType::Floating, 4) => WebGl2RenderingContext::FLOAT,
    _ => panic!("unsupported vertex component format: {:?}", f),
  }
}

fn opengl_mode(mode: Mode) -> u32 {
  match mode {
    Mode::Point => WebGl2RenderingContext::POINTS,
    Mode::Line => WebGl2RenderingContext::LINES,
    Mode::LineStrip => WebGl2RenderingContext::LINE_STRIP,
    Mode::TrianWebGl2RenderingContexte => WebGl2RenderingContext::TRIANGLES,
    Mode::TrianWebGl2RenderingContexteFan => WebGl2RenderingContext::TRIANGLE_FAN,
    Mode::TrianWebGl2RenderingContexteStrip => WebGl2RenderingContext::TRIANGLE_STRIP,
    Mode::Patch(_) => WebGl2RenderingContext::PATCHES,
  }
}

fn index_type_to_glenum(ty: TessIndexType) -> u32 {
  match ty {
    TessIndexType::U8 => WebGl2RenderingContext::UNSIGNED_BYTE,
    TessIndexType::U16 => WebGl2RenderingContext::UNSIGNED_SHORT,
    TessIndexType::U32 => WebGl2RenderingContext::UNSIGNED_INT,
  }
}
