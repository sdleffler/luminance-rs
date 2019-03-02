extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::fmt;
use syn::{
  self, Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Lit, Meta,
  NestedMeta, Type, parse_macro_input
};
use syn::parse::Parse;

#[proc_macro_derive(Vertex, attributes(vertex))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => {
      match generate_struct_vertex_impl(di.ident, di.attrs.iter(), struct_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only structs are currently supported for deriving Vertex")
  }
}

#[proc_macro_derive(VertexAttribSem, attributes(sem))]
pub fn derive_vertex_attrib_sem(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle enums
    Data::Enum(enum_) => {
      match generate_enum_vertex_attrib_sem_impl(di.ident, enum_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only enums are currently supported for deriving VertexAttribSem")
  }
}

#[derive(Debug)]
enum StructImplError {
  SemanticsError(AttrError),
  UnsupportedUnnamed,
  UnsupportedUnit,
  FieldsError(Vec<FieldError>)
}

impl fmt::Display for StructImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StructImplError::SemanticsError(ref e) =>
        write!(f, "error with semantics type; {}", e),
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
fn generate_struct_vertex_impl<'a, A>(
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
      let fields = named_fields.named.into_iter().map(get_field_type_semantics);
      let mut indexed_vertex_attrib_fmts = Vec::new();
      let mut fields_tys = Vec::new();
      let mut errors = Vec::new();

      // partition and generate IndexedVertexAttribFmt
      for r in fields {
        match r {
          Ok((ty, semantics)) => {
            let indexed_vertex_attrib_fmt_q = quote!{
              luminance::vertex::IndexedVertexAttribFmt::new::<#sem_type>(
                #semantics,
                <#ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_FMT
              )
            };

            indexed_vertex_attrib_fmts.push(indexed_vertex_attrib_fmt_q);
            fields_tys.push(ty);
          }

          Err(err) => errors.push(err)
        }
      }

      if !errors.is_empty() {
        return Err(StructImplError::FieldsError(errors));
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
      FieldError::SemanticsParseError(ref e) => write!(f, "unable to parse semantics: {}", e),
      FieldError::AttributeError(ref e) => write!(f, "{}", e)
    }
  }
}

fn get_field_type_semantics(field: Field) -> Result<(syn::Type, Expr), FieldError> {
  let field_ident = field.ident.unwrap();

  let semantics = get_field_attr_once(&field_ident, field.attrs.iter(), "vertex", "sem")
    .map_err(FieldError::AttributeError)?;

  Ok((field.ty.clone(), semantics))
}

#[derive(Debug)]
enum AttrError {
  Several(Ident, String, String),
  CannotFindAttribute(Ident, String, String),
  CannotParseAttribute(Ident, String, String)
}

impl fmt::Display for AttrError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      AttrError::Several(ref field, ref key, ref sub_key) =>
        write!(f, "expected one pair {}({} = …) for {}, got several", key, sub_key, field),
      AttrError::CannotFindAttribute(ref field, ref key, ref sub_key) =>
        write!(f, "no attribute found {}({} = …) for {}", key, sub_key, field),
      AttrError::CannotParseAttribute(ref field, ref key, ref sub_key) =>
        write!(f, "cannot parse attribute {}({} = …) for {}", key, sub_key, field),
    }
  }
}

/// Get and parse an attribute on a field or a variant that must appear only once with the following
/// syntax:
///
///   #[key(sub_key = "lit")]
fn get_field_attr_once<'a, A, T>(
  field_ident: &Ident,
  attrs: A,
  key: &str,
  sub_key: &str
) -> Result<T, AttrError> where A: IntoIterator<Item = &'a Attribute>, T: Parse {
  let mut lit = None;

  for attr in attrs.into_iter() {
    match attr.parse_meta() {
      Ok(Meta::List(ref ml)) if ml.ident == key => {
        let nested = &ml.nested;

        for nested in nested.into_iter() {
          match nested {
            NestedMeta::Meta(Meta::NameValue(ref mnv)) if mnv.ident == sub_key => {
              if lit.is_some() {
                return Err(AttrError::Several(field_ident.clone(), key.to_owned(), sub_key.to_owned()));
              }

              if let Lit::Str(ref strlit) = mnv.lit {
                lit = Some(strlit.parse().map_err(|_| AttrError::CannotParseAttribute(field_ident.clone(), key.to_owned(), sub_key.to_owned()))?);
              }
            }

            _ => ()
          }
        }

      }

      _ => () // ignore things that might not be ours
    }
  }

  lit.ok_or(AttrError::CannotFindAttribute(field_ident.clone(), key.to_owned(), sub_key.to_owned()))
}

#[derive(Debug)]
enum VertexAttribSemImplError {
  AttributeErrors(Vec<AttrError>),
}

impl fmt::Display for VertexAttribSemImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      VertexAttribSemImplError::AttributeErrors(ref errs) => {
        for err in errs {
          err.fmt(f)?;
          writeln!(f, "").unwrap();
        }

        Ok(())
      }
    }
  }
}

/// Get vertex semantics attributes.
///
///   (name, repr, type_name)
fn get_vertex_sem_attribs<'a, A>(var_name: &Ident, attrs: A) -> Result<(Ident, Type, Type), AttrError> where A: IntoIterator<Item = &'a Attribute> + Clone {
  let sem_name = get_field_attr_once::<_, Ident>(var_name, attrs.clone(), "sem", "name")?;
  let sem_repr = get_field_attr_once::<_, Type>(var_name, attrs.clone(), "sem", "repr")?;
  let sem_type_name = get_field_attr_once::<_, Type>(var_name, attrs, "sem", "type_name")?;

  Ok((sem_name, sem_repr, sem_type_name))
}

fn generate_enum_vertex_attrib_sem_impl(ident: Ident, enum_: DataEnum) -> Result<TokenStream, VertexAttribSemImplError> {
  let fields = enum_.variants.into_iter().map(|var| {
    get_vertex_sem_attribs(&var.ident, var.attrs.iter()).map(|attrs| {
      (var.ident, attrs.0, attrs.1, attrs.2)
    })
  });

  // we partition the variants so that we get the ones errored
  let mut parse_branches = Vec::new();
  let mut repr_ty_names = Vec::new();
  let mut ty_names = Vec::new();
  let mut errors = Vec::new();

  for field in fields {
    match field {
      Ok(field) => {
        // parse branches
        let sem_var = field.0;
        let sem_name = field.1.to_string();
        let branch = quote!{
          #sem_name => Some(#ident::#sem_var)
        };

        parse_branches.push(branch);

        // repr type names & type names
        repr_ty_names.push(field.2);
        ty_names.push(field.3);
      }

      Err(e) => errors.push(e)
    }
  }

  if !errors.is_empty() {
    return Err(VertexAttribSemImplError::AttributeErrors(errors));
  }

  // generate the implementation of VertexAttribSem
  let vertex_attrib_sem_impl = quote!{
    impl luminance::vertex::VertexAttribSem for #ident {
      fn index(&self) -> usize {
        *self as usize
      }

      fn parse(name: &str) -> Option<Self> {
        match name {
          #(#parse_branches,)*
          _ => None
        }
      }
    }
  };

  // generate the repr types
  let generated_types = ty_names.into_iter().zip(repr_ty_names).map(|(ty_name, repr_ty_name)| {
    quote!{
      pub struct #ty_name {
        repr: #repr_ty_name
      }

      impl From<#repr_ty_name> for #ty_name {
        fn from(repr: #repr_ty_name) -> Self {
          #ty_name {
            repr
          }
        }
      }

      impl #ty_name {
        pub fn new(repr: #repr_ty_name) -> Self {
          repr.into()
        }
      }
    }
  });

  let output = quote!{
    #vertex_attrib_sem_impl
    #(#generated_types)*
  };

  Ok(output.into())
}
