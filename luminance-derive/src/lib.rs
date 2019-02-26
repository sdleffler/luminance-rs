extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::fmt;
use syn::{
  self, Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Lit, Meta,
  NestedMeta, parse_macro_input
};

const VERTEX_ATTR_KEY: &str = "vertex";
const SEMANTICS_ATTR_KEY: &str = "semantics";

#[proc_macro_derive(Vertex, attributes(vertex))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => {
      match generate_struct_vertex_impl(di.ident, struct_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only structs are currently supported for deriving Vertex")
  }
}

//#[proc_macro_derive(VertexAttribSem, attributes(attrib))]
//pub fn derive_vertex_attrib_sem(input: TokenStream) -> TokenStream {
//  let di: DeriveInput = parse_macro_input!(input);
//
//  match di.data {
//    // for now, we only handle enums
//    Data::Enum(enum_) => {
//      match generate_enum_vertex_attrib_sem_impl(di.ident, enum_) {
//        Ok(impl_) => impl_,
//        Err(e) => panic!("{}", e)
//      }
//    }
//
//    _ => panic!("only enums are currently supported for deriving VertexAttribSem")
//  }
//}

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
              luminance::vertex::IndexedVertexAttribFmt::new(
                #semantics,
                <#ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_FMT
              )
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
        unsafe impl<'a> luminance::vertex::Vertex<'a> for #struct_name {
          type Deinterleaved = (#(&'a [#fields_tys]),*);

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

#[derive(Debug)]
enum FieldError {
  SemanticsParseError(syn::Error),
  AttributeError(AttrError),
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
      FieldError::AttributeError(ref e) => write!(f, "{}", e)
    }
  }
}

fn get_field_type_semantics(field: Field) -> Result<(syn::Type, Expr), FieldError> {
  let field_ident = field.ident.unwrap();

  let lit = get_field_attr_once(&field_ident, field.attrs, VERTEX_ATTR_KEY, SEMANTICS_ATTR_KEY)
    .map_err(FieldError::AttributeError)?;

  match lit {
    Lit::Str(ref semantics) => Ok((field.ty.clone(), semantics.parse()?)),
    _ => Err(FieldError::AttributeError(AttrError::WrongFormat(field_ident)))
  }
}

#[derive(Debug)]
enum AttrError {
  WrongFormat(Ident),
  Several(Ident, String, String),
  CannotFindAttribute(Ident, String, String)
}

impl fmt::Display for AttrError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      AttrError::WrongFormat(ref field) => write!(f, "wrong attribute format for field {}", field),
      AttrError::Several(ref field, ref key, ref sub_key) =>
        write!(f, "expected one pair {}({} = …) for field {}, got several", key, sub_key, field),
      AttrError::CannotFindAttribute(ref field, ref key, ref sub_key) =>
        write!(f, "no attribute found {}({} = …) for field {}", key, sub_key, field)
    }
  }
}

/// Get an attribute on a field or a variant that must appear only once with the following syntax:
///
///   #[key(sub_key = lit)]
///
/// The literal value is free to inspection.
fn get_field_attr_once<A>(
  field_ident: &Ident,
  attrs: A,
  key: &str,
  sub_key: &str
) -> Result<Lit, AttrError> where A: IntoIterator<Item = Attribute> {
  let mut lit = None;

  for attr in attrs.into_iter() {
    match attr.parse_meta() {
      Ok(Meta::List(ref ml)) if ml.ident == key => {
        let nested = &ml.nested;

        if nested.len() != 1 {
          return Err(AttrError::WrongFormat(field_ident.clone()));
        }

        match nested.into_iter().next().unwrap() {
          NestedMeta::Meta(Meta::NameValue(ref mnv)) if mnv.ident == sub_key => {
            if lit.is_some() {
              return Err(AttrError::Several(field_ident.clone(), key.to_owned(), sub_key.to_owned()));
            }

            lit = Some(mnv.lit.clone());
          }

          _ => ()
        }
      }

      _ => () // ignore things that might not be ours
    }
  }

  lit.ok_or(AttrError::CannotFindAttribute(field_ident.clone(), key.to_owned(), sub_key.to_owned()))
}

//fn generate_enum_vertex_attrib_sem_impl(ident: Ident, enum_: DataEnum) -> Result<TokenStream, ()> {
//  let variants_names =
//    enum_.variants.into_iter().map(|var| {
//      for attr in var.attrs.into_iter() {
//      }
//    });
//
//  Err(())
//}
