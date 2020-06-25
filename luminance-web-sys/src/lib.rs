use luminance_windowing::{WindowDim, WindowOpt};
use std::fmt;
use wasm_bindgen::JsValue;
use web_sys::{Document, HtmlCanvasElement, WebGl2RenderingContext, Window};

/// web-sys errors that might occur while initializing and using the platform.
#[non_exhaustive]
#[derive(Debug)]
pub enum WebSysWebGL2SurfaceError {
  CannotGrabWindow,
  CannotGrabDocument,
  NotSuchCanvasElement(String),
  CannotGrabWebGL2Context,
  NoAvailableWebGL2Context,
}

impl WebSysWebGL2SurfaceError {
  fn cannot_grab_window() -> Self {
    WebSysWebGL2SurfaceError::CannotGrabWindow
  }

  fn cannot_grab_document() -> Self {
    WebSysWebGL2SurfaceError::CannotGrabDocument
  }

  fn not_such_canvas_element(name: impl Into<String>) -> Self {
    WebSysWebGL2SurfaceError::NotSuchCanvasElement(name.into())
  }

  fn cannot_grab_webgl2_context() -> Self {
    WebSysWebGL2SurfaceError::CannotGrabWebGL2Context
  }

  fn no_available_webgl2_context() -> Self {
    WebSysWebGL2SurfaceError::NoAvailableWebGL2Context
  }
}

impl fmt::Display for WebSysWebGL2SurfaceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      WebSysWebGL2SurfaceError::CannotGrabWindow => f.write_str("cannot grab the window node"),
      WebSysWebGL2SurfaceError::CannotGrabDocument => f.write_str("cannot grab the document node"),
      WebSysWebGL2SurfaceError::NotSuchCanvasElement(ref name) => {
        write!(f, "cannot grab canvas named {}", name)
      }
      WebSysWebGL2SurfaceError::CannotGrabWebGL2Context => {
        f.write_str("cannot grab WebGL2 context")
      }
      WebSysWebGL2SurfaceError::NoAvailableWebGL2Context => {
        f.write_str("no available WebGL2 context")
      }
    }
  }
}

impl std::error::Error for WebSysWebGL2SurfaceError {}

/// web-sys surface for WebGL2.
pub struct WebSysWebGL2Surface {
  window: Window,
  document: Document,
  canvas: HtmlCanvasElement,
  context: WebGl2RenderingContext,
}

impl WebSysWebGL2Surface {
  pub fn new(
    canvas_name: &str,
    dim: WindowDim,
    title: impl AsRef<str>,
    win_opt: WindowOpt,
  ) -> Result<Self, WebSysWebGL2SurfaceError> {
    let window = web_sys::window().ok_or_else(|| WebSysWebGL2SurfaceError::cannot_grab_window())?;

    let document = window
      .document()
      .ok_or_else(|| WebSysWebGL2SurfaceError::cannot_grab_document())?;

    let canvas = document
      .get_element_by_id(canvas_name)
      .ok_or_else(|| WebSysWebGL2SurfaceError::not_such_canvas_element(canvas_name))?;
    let canvas: JsValue = canvas.into();
    let canvas: web_sys::HtmlCanvasElement = canvas.into();

    match dim {
      WindowDim::Windowed { width, height } | WindowDim::FullscreenRestricted { width, height } => {
        canvas.set_width(width);
        canvas.set_height(height);
      }

      WindowDim::Fullscreen => todo!("fullscreen mode not available yet"),
    }

    let webgl2 = canvas
      .get_context("webgl2")
      .map_err(|_| WebSysWebGL2SurfaceError::CannotGrabWebGL2Context)?
      .ok_or_else(|| WebSysWebGL2SurfaceError::NoAvailableWebGL2Context)?;
    let webgl2: &JsValue = webgl2.as_ref();
    let webgl2 = webgl2.clone();
    let context: web_sys::WebGl2RenderingContext = webgl2.into();

    Ok(Self {
      window,
      document,
      canvas,
      context,
    })
  }
}
