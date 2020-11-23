use syn::punctuated::Punctuated;

use crate::internal::check;
use crate::internal::Attributes;
use crate::internal::Context;
use crate::internal::Derive;
use crate::internal::FieldAttrs;
use crate::internal::VariantAttrs;

pub struct Container<'a> {
  pub ident: syn::Ident,
  pub attrs: Attributes,
  pub data: Data<'a>,
  pub generics: &'a syn::Generics,
  pub original: &'a syn::DeriveInput,
}

impl<'a> Container<'a> {
  pub fn from_ast(context: &Context, item: &'a syn::DeriveInput, derive: Derive) -> Option<Self> {
    let attrs: Attributes = Attributes::from_ast(context, item);

    let data: Data<'a> = match item.data {
      syn::Data::Enum(ref data) => Data::Enum(expand_enum(context, &data.variants)),
      syn::Data::Struct(ref data) => {
        let (style, fields): _ = expand_struct(context, &data.fields);
        Data::Struct(style, fields)
      }
      syn::Data::Union(_) => {
        context.error_spanned_by(item, "Cannot derive Diff/Merge for unions");
        return None;
      }
    };

    let mut this: Self = Self {
      ident: item.ident.clone(),
      attrs,
      data,
      generics: &item.generics,
      original: item,
    };

    check::check(context, &mut this, derive);

    Some(this)
  }
}

fn expand_struct<'a>(context: &Context, fields: &'a syn::Fields) -> (Style, Vec<Field<'a>>) {
  match fields {
    syn::Fields::Named(ref fields) => (Style::Struct, expand_fields(context, &fields.named)),
    syn::Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => {
      (Style::Newtype, expand_fields(context, &fields.unnamed))
    }
    syn::Fields::Unnamed(ref fields) => (Style::Tuple, expand_fields(context, &fields.unnamed)),
    syn::Fields::Unit => (Style::Unit, Vec::new()),
  }
}

fn expand_enum<'a>(
  context: &Context,
  variants: &'a Punctuated<syn::Variant, Token![,]>,
) -> Vec<Variant<'a>> {
  variants
    .iter()
    .map(|variant| Variant::from_ast(context, variant))
    .collect()
}

fn expand_fields<'a>(
  context: &Context,
  fields: &'a Punctuated<syn::Field, Token![,]>,
) -> Vec<Field<'a>> {
  fields
    .iter()
    .enumerate()
    .map(|(index, field)| Field::from_ast(context, index, field))
    .collect()
}

pub enum Data<'a> {
  Enum(Vec<Variant<'a>>),
  Struct(Style, Vec<Field<'a>>),
}

pub struct Variant<'a> {
  pub ident: syn::Ident,
  pub attrs: VariantAttrs,
  pub style: Style,
  pub fields: Vec<Field<'a>>,
  pub original: &'a syn::Variant,
}

impl<'a> Variant<'a> {
  pub fn from_ast(context: &Context, variant: &'a syn::Variant) -> Self {
    let attrs: VariantAttrs = VariantAttrs::from_ast(context, variant);
    let (style, fields): _ = expand_struct(context, &variant.fields);

    Self {
      ident: variant.ident.clone(),
      attrs,
      style,
      fields,
      original: variant,
    }
  }

  pub fn effective_style(&self) -> Style {
    if matches!(self.style, Style::Newtype if self.fields[0].attrs.skip_diff()) {
      Style::Unit
    } else {
      self.style
    }
  }
}

pub struct Field<'a> {
  pub member: syn::Member,
  pub attrs: FieldAttrs,
  pub ty: &'a syn::Type,
  pub original: &'a syn::Field,
}

impl<'a> Field<'a> {
  pub fn from_ast(context: &Context, index: usize, field: &'a syn::Field) -> Self {
    Self {
      member: match field.ident {
        Some(ref ident) => syn::Member::Named(ident.clone()),
        None => syn::Member::Unnamed(index.into()),
      },
      attrs: FieldAttrs::from_ast(context, index, field),
      ty: &field.ty,
      original: field,
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum Style {
  Struct,
  Tuple,
  Newtype,
  Unit,
}
