use crate::attrib::{AttrError, get_field_attr_once};
use proc_macro::TokenStream;
use quote::quote;
use std::fmt;
use syn::{Attribute, DataStruct, Fields, Ident, LitBool, Type};

#[derive(Debug)]
pub(crate) enum StructImplError {
  SemanticsError(AttrError),
  FieldError(AttrError),
  UnsupportedUnnamed,
  UnsupportedUnit,
}

impl fmt::Display for StructImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StructImplError::SemanticsError(ref e) =>
        write!(f, "error with semantics type; {}", e),
      StructImplError::FieldError(ref e) =>
        write!(f, "error with vertex attribute field; {}", e),
      StructImplError::UnsupportedUnnamed => f.write_str("unsupported unnamed fields"),
      StructImplError::UnsupportedUnit => f.write_str("unsupported unit struct"),
    }
  }
}

/// Generate the Vertex impl for a struct.
pub(crate) fn generate_vertex_impl<'a, A>(
  ident: Ident,
  attrs: A,
  struct_: DataStruct
) -> Result<TokenStream, StructImplError>
where A: IntoIterator<Item = &'a Attribute> {
  // search the semantics name
  let sem_type: Type = get_field_attr_once(
    &ident,
    attrs,
    "vertex",
    "sem"
  ).map_err(StructImplError::SemanticsError)?;

  match struct_.fields {
    Fields::Named(named_fields) => {
      let mut indexed_vertex_attrib_fmts = Vec::new();
      let mut fields_tys = Vec::new();

      // partition and generate IndexedVertexAttribFmt
      for field in named_fields.named {
        let instancing_attr = get_field_attr_once(
          field.ident.as_ref().unwrap(),
          field.attrs.iter(),
          "vertex",
          "instanced"
        );
        let instancing = instancing_attr
          .map(|b: LitBool| {
            if b.value {
              quote! { luminance::vertex::VertexInstancing::On }
            } else {
              quote! { luminance::vertex::VertexInstancing::Off }
            }
          })
          .or_else(|e| match e {
            AttrError::CannotFindAttribute(..) => {
              Ok(quote! { luminance::vertex::VertexInstancing::Off })
            }

            _ => Err(e)
          })
          .map_err(StructImplError::FieldError)?;

        let field_ty = field.ty;
        let indexed_vertex_attrib_fmt_q = quote!{
          luminance::vertex::IndexedVertexAttribFmt::new::<#sem_type>(
            <#field_ty as luminance::vertex::HasSemantics>::VERTEX_ATTRIB_SEM,
            #instancing,
            <#field_ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_FMT
          )
        };

        indexed_vertex_attrib_fmts.push(indexed_vertex_attrib_fmt_q);
        fields_tys.push(field_ty);
      }

      // indexed_vertex_attrib_fmts contains the exhaustive list of the indexed vertex attribute
      // formats needed to implement the Vertex trait
      let struct_name = ident;
      let impl_ = quote! {
        unsafe impl<'a> luminance::vertex::Vertex<'a> for #struct_name {
          fn vertex_fmt() -> luminance::vertex::VertexFmt {
            vec![#(#indexed_vertex_attrib_fmts),*]
          }
        }
      };

      Ok(impl_.into())
    }

    Fields::Unnamed(_) => Err(StructImplError::UnsupportedUnnamed),
    Fields::Unit => Err(StructImplError::UnsupportedUnit)
  }
}
