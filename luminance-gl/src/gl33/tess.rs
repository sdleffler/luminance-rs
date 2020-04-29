use gl;
use gl::types::*;
use std::cell::RefCell;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;

use crate::gl33::buffer::{Buffer, BufferSlice, BufferSliceMut};
use crate::gl33::state::{Bind, GLState};
use crate::gl33::vertex_restart::VertexRestart;
use crate::gl33::GL33;
use luminance::backend::buffer::{Buffer as _, BufferSlice as BufferSliceBackend};
use luminance::backend::tess::Tess as TessBackend;
use luminance::tess::{Mode, TessError, TessIndex, TessIndexType, TessMapError};
use luminance::vertex::{
  Normalized, Vertex, VertexAttribDesc, VertexAttribDim, VertexAttribType, VertexBufferDesc,
  VertexDesc, VertexInstancing,
};

#[derive(Debug)]
struct VertexBuffer<T> {
  /// Indexed format of the buffer.
  fmt: VertexDesc,
  /// Internal buffer.
  buf: Buffer<T>,
}

pub struct Tess<V, I, W>
where
  V: Vertex,
  I: TessIndex,
  W: Vertex,
{
  mode: GLenum,
  vert_nb: usize,
  inst_nb: usize,
  patch_vert_nb: usize,
  vao: GLenum,
  vertex_buffer: Option<VertexBuffer<V>>,
  index_state: Option<IndexedDrawState<I>>,
  instance_buffer: Option<VertexBuffer<W>>,
  state: Rc<RefCell<GLState>>,
}

impl<V, I, W> Drop for Tess<V, I, W>
where
  V: Vertex,
  I: TessIndex,
  W: Vertex,
{
  fn drop(&mut self) {
    unsafe {
      self.state.borrow_mut().unbind_vertex_array();
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

/// All the extra data required when doing indexed drawing.
struct IndexedDrawState<I>
where
  I: TessIndex,
{
  buffer: Buffer<I>,
  restart_index: Option<I>,
}

unsafe impl<V, I, W> TessBackend<V, I, W> for GL33
where
  V: Vertex,
  I: TessIndex,
  W: Vertex,
{
  type TessRepr = Tess<V, I, W>;

  type VertexSliceRepr = BufferSlice<V>;
  type VertexSliceMutRepr = BufferSliceMut<V>;

  type IndexSliceRepr = BufferSlice<I>;
  type IndexSliceMutRepr = BufferSliceMut<I>;

  type InstanceSliceRepr = BufferSlice<W>;
  type InstanceSliceMutRepr = BufferSliceMut<W>;

  unsafe fn build(
    &mut self,
    vertex_data: Vec<V>,
    index_data: Vec<I>,
    instance_data: Vec<W>,
    mode: Mode,
    vert_nb: usize,
    inst_nb: usize,
    restart_index: Option<I>,
  ) -> Result<Self::TessRepr, TessError> {
    // try to deduce the number of vertices to render if it’s not specified
    let vert_nb = guess_vert_nb_or_fail::<V, I>(&vertex_data, &index_data, vert_nb)?;
    let inst_nb = guess_inst_nb_or_fail::<W>(&instance_data, vert_nb, inst_nb)?;

    let mut vao: GLuint = 0;

    let patch_vert_nb = match mode {
      Mode::Patch(nb) => nb,
      _ => 0,
    };

    gl::GenVertexArrays(1, &mut vao);

    // force binding the vertex array so that previously bound vertex arrays (possibly the same
    // handle) don’t prevent us from binding here
    self.state.borrow_mut().bind_vertex_array(vao, Bind::Forced);

    let vertex_buffer = if !vertex_data.is_empty() {
      let vb = VertexBuffer {
        fmt: V::vertex_desc(),
        buf: self.from_vec(vertex_data)?,
      };

      // force binding as it’s meaningful when a vao is bound
      self
        .state
        .borrow_mut()
        .bind_array_buffer(vb.buf.gl_buf, Bind::Forced);
      set_vertex_pointers(&vb.fmt);

      Some(vb)
    } else {
      None
    };

    // in case of indexed render, create an index buffer
    let index_state = if !index_data.is_empty() {
      let state = IndexedDrawState {
        restart_index,
        buffer: self.from_vec(index_data)?,
      };

      // force binding as it’s meaningful when a vao is bound
      self
        .state
        .borrow_mut()
        .bind_element_array_buffer(state.buffer.gl_buf, Bind::Forced);

      Some(state)
    } else {
      None
    };

    let instance_buffer = if !instance_data.is_empty() {
      let ib = VertexBuffer {
        fmt: W::vertex_desc(),
        buf: self.from_vec(instance_data)?,
      };

      // force binding as it’s meaningful when a vao is bound
      self
        .state
        .borrow_mut()
        .bind_array_buffer(ib.buf.gl_buf, Bind::Forced);
      set_vertex_pointers(&ib.fmt);

      Some(ib)
    } else {
      None
    };

    // convert to OpenGL-friendly internals and return
    Ok(Tess {
      mode: opengl_mode(mode),
      vert_nb,
      inst_nb,
      patch_vert_nb,
      vao,
      vertex_buffer,
      instance_buffer,
      index_state,
      state: self.state.clone(),
    })
  }

  unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize {
    tess.vert_nb
  }

  unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize {
    tess.inst_nb
  }

  unsafe fn render(
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError> {
    let vert_nb = vert_nb as GLsizei;
    let inst_nb = inst_nb as GLsizei;

    let mut gfx_st = tess.state.borrow_mut();
    gfx_st.bind_vertex_array(tess.vao, Bind::Cached);

    if tess.mode == gl::PATCHES {
      gfx_st.set_patch_vertex_nb(tess.patch_vert_nb);
    }

    if let Some(index_state) = tess.index_state.as_ref() {
      // indexed render
      let first = (I::INDEX_TYPE.bytes() * start_index) as *const c_void;

      if let Some(restart_index) = index_state.restart_index {
        gfx_st.set_vertex_restart(VertexRestart::On);
        gl::PrimitiveRestartIndex(restart_index.into());
      } else {
        gfx_st.set_vertex_restart(VertexRestart::Off);
      }

      if inst_nb <= 1 {
        gl::DrawElements(
          tess.mode,
          vert_nb,
          index_type_to_glenum(I::INDEX_TYPE),
          first,
        );
      } else {
        gl::DrawElementsInstanced(
          tess.mode,
          vert_nb,
          index_type_to_glenum(I::INDEX_TYPE),
          first,
          inst_nb,
        );
      }
    } else {
      // direct render
      let first = start_index as GLint;

      if inst_nb <= 1 {
        gl::DrawArrays(tess.mode, first, vert_nb);
      } else {
        gl::DrawArraysInstanced(tess.mode, first, vert_nb, inst_nb);
      }
    }

    Ok(())
  }

  unsafe fn vertices(tess: &mut Self::TessRepr) -> Result<Self::VertexSliceRepr, TessMapError> {
    match tess.vertex_buffer {
      None => Err(TessMapError::ForbiddenAttributelessMapping),

      Some(ref vb) => GL33::slice_buffer(&vb.buf).map_err(TessMapError::BufferMapError),
    }
  }

  unsafe fn vertices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::VertexSliceMutRepr, TessMapError> {
    match tess.vertex_buffer {
      None => Err(TessMapError::ForbiddenAttributelessMapping),

      Some(ref mut vb) => GL33::slice_buffer_mut(&mut vb.buf).map_err(TessMapError::BufferMapError),
    }
  }

  unsafe fn indices(tess: &mut Self::TessRepr) -> Result<Self::IndexSliceRepr, TessMapError> {
    match tess.index_state {
      None => Err(TessMapError::ForbiddenAttributelessMapping),

      Some(ref vb) => GL33::slice_buffer(&vb.buffer).map_err(TessMapError::BufferMapError),
    }
  }

  unsafe fn indices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::IndexSliceMutRepr, TessMapError> {
    match tess.index_state {
      None => Err(TessMapError::ForbiddenAttributelessMapping),

      Some(ref mut vb) => {
        GL33::slice_buffer_mut(&mut vb.buffer).map_err(TessMapError::BufferMapError)
      }
    }
  }

  unsafe fn instances(tess: &mut Self::TessRepr) -> Result<Self::InstanceSliceRepr, TessMapError> {
    match tess.instance_buffer {
      None => Err(TessMapError::ForbiddenAttributelessMapping),

      Some(ref vb) => GL33::slice_buffer(&vb.buf).map_err(TessMapError::BufferMapError),
    }
  }

  unsafe fn instances_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceMutRepr, TessMapError> {
    match tess.instance_buffer {
      None => Err(TessMapError::ForbiddenAttributelessMapping),

      Some(ref mut vb) => GL33::slice_buffer_mut(&mut vb.buf).map_err(TessMapError::BufferMapError),
    }
  }
}

// Give OpenGL types information on the content of the VBO by setting vertex descriptors and pointers
// to buffer memory.
fn set_vertex_pointers(descriptors: &[VertexBufferDesc]) {
  // this function sets the vertex attribute pointer for the input list by computing:
  //   - The vertex attribute ID: this is the “rank” of the attribute in the input list (order
  //     matters, for short).
  //   - The stride: this is easily computed, since it’s the size (bytes) of a single vertex.
  //   - The offsets: each attribute has a given offset in the buffer. This is computed by
  //     accumulating the size of all previously set attributes.
  let offsets = aligned_offsets(descriptors);
  let vertex_weight = offset_based_vertex_weight(descriptors, &offsets) as GLsizei;

  for (desc, off) in descriptors.iter().zip(offsets) {
    set_component_format(vertex_weight, off, desc);
  }
}

// Compute offsets for all the vertex components according to the alignments provided.
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

// Align an offset.
#[inline]
fn off_align(off: usize, align: usize) -> usize {
  let a = align - 1;
  (off + a) & !a
}

// Weight in bytes of a vertex component.
fn component_weight(f: &VertexAttribDesc) -> usize {
  dim_as_size(f.dim) as usize * f.unit_size
}

fn dim_as_size(d: VertexAttribDim) -> GLint {
  match d {
    VertexAttribDim::Dim1 => 1,
    VertexAttribDim::Dim2 => 2,
    VertexAttribDim::Dim3 => 3,
    VertexAttribDim::Dim4 => 4,
  }
}

// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
// aligned.
fn offset_based_vertex_weight(descriptors: &[VertexBufferDesc], offsets: &[usize]) -> usize {
  if descriptors.is_empty() || offsets.is_empty() {
    return 0;
  }

  off_align(
    offsets[offsets.len() - 1] + component_weight(&descriptors[descriptors.len() - 1].attrib_desc),
    descriptors[0].attrib_desc.align,
  )
}

// Set the vertex component OpenGL pointers regarding the index of the component (i), the stride
fn set_component_format(stride: GLsizei, off: usize, desc: &VertexBufferDesc) {
  let attrib_desc = &desc.attrib_desc;
  let index = desc.index as GLuint;

  unsafe {
    match attrib_desc.ty {
      VertexAttribType::Floating => {
        gl::VertexAttribPointer(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          gl::FALSE,
          stride,
          ptr::null::<c_void>().add(off),
        );
      }

      VertexAttribType::Integral(Normalized::No)
      | VertexAttribType::Unsigned(Normalized::No)
      | VertexAttribType::Boolean => {
        // non-normalized integrals / booleans
        gl::VertexAttribIPointer(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          stride,
          ptr::null::<c_void>().add(off),
        );
      }

      _ => {
        // normalized integrals
        gl::VertexAttribPointer(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          gl::TRUE,
          stride,
          ptr::null::<c_void>().add(off),
        );
      }
    }

    // set vertex attribute divisor based on the vertex instancing configuration
    let divisor = match desc.instancing {
      VertexInstancing::On => 1,
      VertexInstancing::Off => 0,
    };
    gl::VertexAttribDivisor(index, divisor);

    gl::EnableVertexAttribArray(index);
  }
}

fn opengl_sized_type(f: &VertexAttribDesc) -> GLenum {
  match (f.ty, f.unit_size) {
    (VertexAttribType::Integral(_), 1) => gl::BYTE,
    (VertexAttribType::Integral(_), 2) => gl::SHORT,
    (VertexAttribType::Integral(_), 4) => gl::INT,
    (VertexAttribType::Unsigned(_), 1) | (VertexAttribType::Boolean, 1) => gl::UNSIGNED_BYTE,
    (VertexAttribType::Unsigned(_), 2) => gl::UNSIGNED_SHORT,
    (VertexAttribType::Unsigned(_), 4) => gl::UNSIGNED_INT,
    (VertexAttribType::Floating, 4) => gl::FLOAT,
    _ => panic!("unsupported vertex component format: {:?}", f),
  }
}

fn opengl_mode(mode: Mode) -> GLenum {
  match mode {
    Mode::Point => gl::POINTS,
    Mode::Line => gl::LINES,
    Mode::LineStrip => gl::LINE_STRIP,
    Mode::Triangle => gl::TRIANGLES,
    Mode::TriangleFan => gl::TRIANGLE_FAN,
    Mode::TriangleStrip => gl::TRIANGLE_STRIP,
    Mode::Patch(_) => gl::PATCHES,
  }
}

fn index_type_to_glenum(ty: TessIndexType) -> GLenum {
  match ty {
    TessIndexType::U8 => gl::UNSIGNED_BYTE,
    TessIndexType::U16 => gl::UNSIGNED_SHORT,
    TessIndexType::U32 => gl::UNSIGNED_INT,
  }
}

/// Guess how many vertices there are to render based on the current configuration or fail if
/// incorrectly configured.
fn guess_vert_nb_or_fail<V, I>(
  vertex_data: &Vec<V>,
  index_data: &Vec<I>,
  vert_nb: usize,
) -> Result<usize, TessError> {
  if vert_nb == 0 {
    // we don’t have an explicit vertex number to render; go and guess!
    if !index_data.is_empty() {
      Ok(index_data.len())
    } else {
      // FIXME: resume this code as soon as we supporte deinterleaved data
      // // deduce the number of vertices based on the vertex data; they all
      // // must be of the same length, otherwise it’s an error
      // match self.vertex_buffers.len() {
      //   0 => Err(TessError::AttributelessError(
      //     "attributeless render with no vertex number".to_owned(),
      //   )),

      //   1 => Ok(self.vertex_buffers[0].buf.len),

      //   _ => {
      //     let vert_nb = self.vertex_buffers[0].buf.len;
      //     let incoherent = Self::check_incoherent_buffers(self.vertex_buffers.iter(), vert_nb);

      //     if incoherent {
      //       Err(TessError::LengthIncoherency(vert_nb))
      //     } else {
      //       Ok(vert_nb)
      //     }
      //   }
      // }
      Ok(vertex_data.len())
    }
  } else {
    // we have an explicit number of vertices to render, but we’re gonna check that number actually
    // makes sense
    if index_data.is_empty() {
      // FIXME
      //let incoherent = Self::check_incoherent_buffers(self.vertex_buffers.iter(), self.vert_nb);
      let incoherent = vertex_data.len() != vert_nb;

      if incoherent {
        return Err(TessError::LengthIncoherency(vert_nb));
      } else if !vertex_data.is_empty() && vertex_data.len() < vert_nb {
        return Err(TessError::Overflow(vertex_data.len(), vert_nb));
      }
    } else {
      // we have indices (indirect draw); so we’ll compare to them
      if index_data.len() < vert_nb {
        return Err(TessError::Overflow(index_data.len(), vert_nb));
      }
    }

    Ok(vert_nb)
  }
}

/// Guess how many instances there are to render based on the current configuration or fail if
/// incorrectly configured.
fn guess_inst_nb_or_fail<W>(
  inst_data: &Vec<W>,
  vert_nb: usize,
  inst_nb: usize,
) -> Result<usize, TessError> {
  if inst_nb == 0 {
    // we don’t have an explicit instance number to render; we depend on inst_data
    Ok(inst_data.len())
  } else {
    // we have an explicit number of instances to render, but we’re gonna check that number
    // actually makes sense
    let incoherent = inst_data.len() != vert_nb;

    if incoherent {
      return Err(TessError::LengthIncoherency(inst_nb));
    } else if !inst_data.is_empty() && inst_data.len() < inst_nb {
      return Err(TessError::Overflow(inst_data.len(), inst_nb));
    }

    Ok(inst_nb)
  }
}
