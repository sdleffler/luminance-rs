extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::fmt;
use syn::{
  self, Data, DataStruct, DeriveInput, Field, Fields, Ident, Lit, Meta, Variant,
  parse_macro_input
};

const SEMANTICS_ATTR_KEY: &str = "semantics";

#[proc_macro_derive(Vertex)]
pub fn vertex(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => {
      match generate_struct_vertex_impl(di.ident, struct_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only structs are currently supported")
  }
}

#[derive(Debug)]
enum StructImplError {
  UnsupportedUnnamed,
  UnsupportedUnit,
  FieldsError(Vec<FieldError>)
}

impl fmt::Display for StructImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StructImplError::UnsupportedUnnamed => f.write_str("unsupported unnamed fields"),
      StructImplError::UnsupportedUnit => f.write_str("unsupported unit struct"),
      StructImplError::FieldsError(ref errs) => {
        for err in errs {
          err.fmt(f)?;
          writeln!(f, "").unwrap();
        }

        Ok(())
      }
    }
  }
}

/// Generate the Vertex impl for a struct.
fn generate_struct_vertex_impl(ident: Ident, struct_: DataStruct) -> Result<TokenStream, StructImplError> {
  match struct_.fields {
    Fields::Named(named_fields) => {
      let fields = named_fields.named.into_iter().map(get_field_type_semantics);
      let mut indexed_vertex_attrib_fmts = Vec::new();
      let mut fields_tys = Vec::new();
      let mut errored = Vec::new();

      // partition and generate IndexedVertexAttribFmt
      for r in fields {
        match r {
          Ok((ty, semantics)) => {
            let indexed_vertex_attrib_fmt_q = quote!{
              IndexedVertexAttribFmt::new(#semantics.index(), #ty::VERTEX_ATTRIB_FMT)
            };

            indexed_vertex_attrib_fmts.push(indexed_vertex_attrib_fmt_q);
            fields_tys.push(ty);
          }

          Err(err) => errored.push(err)
        }
      }

      if !errored.is_empty() {
        return Err(StructImplError::FieldsError(errored));
      }

      // indexed_vertex_attrib_fmts contains the exhaustive list of the indexed vertex attribute
      // formats needed to implement the Vertex trait
      let struct_name = ident;
      let impl_ = quote! {
        impl<'a> Vertex<'a> for #struct_name {
          type Deinterleaved = &'a (#(#fields_tys),*);

          const VERTEX_FMT: VertexFmt = &[#(#indexed_vertex_attrib_fmts),*];
        }
      };

      Ok(impl_.into())
    }

    Fields::Unnamed(_) => Err(StructImplError::UnsupportedUnnamed),
    Fields::Unit => Err(StructImplError::UnsupportedUnit)
  }
}

#[derive(Debug)]
enum FieldError {
  SemanticsParseError(syn::Error),
  SeveralSemantics(Ident),
  WrongSemanticsFormat(Ident),
  SemanticsKeyNotFound(Ident)
}

impl From<syn::Error> for FieldError {
  fn from(e: syn::Error) -> Self {
    FieldError::SemanticsParseError(e)
  }
}

impl fmt::Display for FieldError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      FieldError::SemanticsParseError(ref e) => write!(f, "unable to pars semantics: {}", e),
      FieldError::SeveralSemantics(ref field) =>
        write!(f, "several semantics annotations were found while expecting one for field {}", field),
      FieldError::WrongSemanticsFormat(ref field) => write!(f, "the semantics should be a variant for field {}", field),
      FieldError::SemanticsKeyNotFound(ref field) => write!(f, "the semantics annotation was not found for field {}", field)
    }
  }
}

fn get_field_type_semantics(field: Field) -> Result<(syn::Type, Variant), FieldError> {
  let mut semantics_found = false;
  let mut ty_semantics = None;
  let field_ident = field.ident.unwrap();

  for attr in field.attrs {
    match attr.parse_meta() {
      Ok(Meta::NameValue(ref mnv)) if mnv.ident == SEMANTICS_ATTR_KEY => {
        match mnv.lit {
          Lit::Str(ref semantics) => {
            if !semantics_found {
              semantics_found = true;
              ty_semantics = Some((field.ty.clone(), semantics.parse()?));
            } else {
              return Err(FieldError::SeveralSemantics(field_ident.clone()));
            }
          }

          _ => return Err(FieldError::WrongSemanticsFormat(field_ident.clone()))
        }
      }

      // we ignore all other metas as it might be stuff from some other crates
      _ => ()
    }
  }

  // here, ty_semantics holds either our type and its associated semantics or we’re missing the
  // semantics key
  ty_semantics.ok_or(FieldError::SemanticsKeyNotFound(field_ident))
}
