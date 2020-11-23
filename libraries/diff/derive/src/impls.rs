use core::fmt::Display;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::borrow::Cow;
use syn::spanned::Spanned as _;
use syn::token::Brace;

use crate::internal::Attributes;
use crate::internal::Container;
use crate::internal::Context;
use crate::internal::Data;
use crate::internal::Derive;
use crate::internal::Field;
use crate::internal::Style;
use crate::internal::TagType;
use crate::internal::Variant;

pub enum Fragment {
  /// Tokens that can be used as an expression.
  Expr(TokenStream),
  /// Tokens that can be used inside a block. The surrounding curly braces are
  /// not part of these tokens.
  Block(TokenStream),
}

impl AsRef<TokenStream> for Fragment {
  fn as_ref(&self) -> &TokenStream {
    match self {
      Self::Expr(ref expr) => expr,
      Self::Block(ref block) => block,
    }
  }
}

macro_rules! quote_expr {
  ($($tt:tt)*) => {
    Fragment::Expr(quote!($($tt)*))
  }
}

macro_rules! quote_block {
  ($($tt:tt)*) => {
    Fragment::Block(quote!($($tt)*))
  }
}

/// Interpolate a fragment as the statements of a block.
pub struct Stmts(pub Fragment);

impl ToTokens for Stmts {
  fn to_tokens(&self, out: &mut TokenStream) {
    match self.0 {
      Fragment::Expr(ref expr) => expr.to_tokens(out),
      Fragment::Block(ref block) => block.to_tokens(out),
    }
  }
}

/// Interpolate a fragment as the value part of a `match` expression. This
/// involves putting a comma after expressions and curly braces around blocks.
pub struct Match(pub Fragment);

impl ToTokens for Match {
  fn to_tokens(&self, out: &mut TokenStream) {
    match self.0 {
      Fragment::Expr(ref expr) => {
        expr.to_tokens(out);
        <Token![,]>::default().to_tokens(out);
      }
      Fragment::Block(ref block) => {
        Brace::default().surround(out, |out| block.to_tokens(out));
      }
    }
  }
}

// =============================================================================
// =============================================================================

pub fn expand_derive_diff(input: &syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
  let context: Context = Context::new();

  let container: Container<'_> = match Container::from_ast(&context, input, Derive::Diff) {
    Some(container) => container,
    None => return Err(context.check().unwrap_err()),
  };

  context.check()?;

  let ident: &syn::Ident = &container.ident;
  let body: Stmts = Stmts(expand_body(ident, &container));
  let diff: Cow<syn::Path> = container.attrs.__lib();

  let generics: syn::Generics = expand_generics(&diff, container.generics.clone());
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let impl_: TokenStream = quote! {
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
      #[allow(rust_2018_idioms, clippy::useless_attribute)]
      extern crate diff as _diff;
      #[automatically_derived]
      impl #impl_generics #diff::Diff for #ident #ty_generics #where_clause {
        fn diff<__D>(lhs: &Self, rhs: &Self, out: __D) -> ::core::result::Result<__D::Ok, __D::Error>
        where
          __D: #diff::Differ,
        {
          #body
        }
      }
    };
  };

  Ok(impl_)
}

fn expand_generics(diff: &syn::Path, mut generics: syn::Generics) -> syn::Generics {
  // TODO: Ignore skipped fields
  // TODO: Ignore PhantomData

  for param in generics.params.iter_mut() {
    if let syn::GenericParam::Type(generic) = param {
      generic.bounds.push(parse_quote!(#diff::Diff));
    }
  }

  generics
}

fn expand_body(this: &syn::Ident, container: &Container) -> Fragment {
  if container.attrs.transparent() {
    expand_transparent(container)
  } else if let Some(into) = container.attrs.type_into() {
    expand_into(into)
  } else {
    match container.data {
      Data::Enum(ref variants) => expand_enum(this, variants, &container.attrs),
      Data::Struct(Style::Struct, ref fields) => expand_struct(fields, &container.attrs),
      Data::Struct(Style::Tuple, ref fields) => expand_tuple_struct(fields, &container.attrs),
      Data::Struct(Style::Newtype, ref fields) => {
        expand_newtype_struct(&fields[0], &container.attrs)
      }
      Data::Struct(Style::Unit, _) => quote_expr!(_diff::Differ::same(out, lhs, rhs)),
    }
  }
}

fn expand_transparent(container: &Container) -> Fragment {
  let fields: &[Field] = match container.data {
    Data::Struct(_, ref fields) => fields,
    Data::Enum(_) => unreachable!(),
  };

  let field: &Field = fields
    .iter()
    .find(|field| field.attrs.transparent())
    .unwrap();

  let member: &syn::Member = &field.member;
  let span: Span = field.original.span();
  let func: TokenStream = quote_spanned!(span=> _diff::Diff::diff);

  quote_block! {
    #func(&lhs.#member, &rhs.#member, out)
  }
}

fn expand_into(into: &syn::Type) -> Fragment {
  quote_block! {
    _diff::Diff::diff(
      &Into::<#into>::into(Clone::clone(lhs)),
      &Into::<#into>::into(Clone::clone(rhs)),
      out,
    )
  }
}

fn expand_struct(fields: &[Field], attrs: &Attributes) -> Fragment {
  assert!(fields.len() as u64 <= u64::from(u32::MAX));

  let stmts: Vec<TokenStream> = expand_struct_fields(fields, false, StructTrait::DiffStruct);
  let size: usize = stmts.len();
  let this: String = attrs.name().diff_name();

  quote_block! {
    let mut out: __D::DiffStruct = _diff::Differ::diff_struct(out, #this, #size);
    #(#stmts)*
    _diff::DiffStruct::end(out)
  }
}

fn expand_tuple_struct(fields: &[Field], attrs: &Attributes) -> Fragment {
  let stmts: Vec<TokenStream> = expand_tuple_fields(fields, false, TupleTrait::DiffTupleStruct);
  let size: usize = stmts.len();
  let this: String = attrs.name().diff_name();

  quote_block! {
    let mut out: __D::DiffTupleStruct = _diff::Differ::diff_tuple_struct(out, #this, #size);
    #(#stmts)*
    _diff::DiffTupleStruct::end(out)
  }
}

fn expand_newtype_struct(field: &Field, attrs: &Attributes) -> Fragment {
  let this: String = attrs.name().diff_name();
  let span: Span = field.original.span();
  let func: TokenStream = quote_spanned!(span=> _diff::Differ::diff_newtype_struct);

  let member: &syn::Member = &field.member;
  let lhs: TokenStream = expand_member(field, quote!(&lhs.#member));
  let rhs: TokenStream = expand_member(field, quote!(&rhs.#member));

  quote_block!(#func(out, #this, #lhs, #rhs))
}

fn expand_enum(this: &syn::Ident, variants: &[Variant], attrs: &Attributes) -> Fragment {
  assert!(variants.len() as u64 <= u64::from(u32::MAX));

  let arms: Vec<TokenStream> = variants
    .iter()
    .map(|variant| expand_variant(this, variant, attrs))
    .collect();

  quote_expr! {
    match (lhs, rhs) {
      #(#arms)*
      _ => _diff::Differ::difference(out, lhs, rhs)
    }
  }
}

fn expand_variant(this: &syn::Ident, variant: &Variant, attrs: &Attributes) -> TokenStream {
  if variant.attrs.skip_diff() {
    let pattern: TokenStream = match variant.style {
      Style::Struct => quote!({ .. }),
      Style::Newtype | Style::Tuple => quote!((..)),
      Style::Unit => quote!(),
    };

    let ident: &syn::Ident = &variant.ident;
    let message: String = format!("invalid enum variant {}::{}", this, ident);
    let error: TokenStream = quote!(Err(_diff::Error::custom(#message)));

    quote!((#this::#ident #pattern, _) | (_, #this::#ident #pattern) => #error)
  } else {
    let case: TokenStream = expand_variant_case(this, variant);
    let this: String = attrs.name().diff_name();
    let name: String = variant.attrs.name().diff_name();

    let body: Match = Match(match attrs.tag() {
      TagType::External => match variant.effective_style() {
        Style::Struct => {
          expand_struct_variant(&variant.fields, this, StructVariant::External { name })
        }
        Style::Tuple => {
          expand_tuple_variant(&variant.fields, TupleVariant::External { name, this })
        }
        Style::Newtype => {
          let field: &Field = &variant.fields[0];
          let span: Span = field.original.span();
          let func: TokenStream = quote_spanned!(span=> _diff::Differ::diff_newtype_variant);

          quote_expr!(#func(out, #this, #name, __lhs0, __rhs0))
        }
        Style::Unit => quote_expr!(_diff::Differ::same(out, lhs, rhs)),
      },
      TagType::Untagged => match variant.effective_style() {
        Style::Struct => expand_struct_variant(&variant.fields, this, StructVariant::Untagged),
        Style::Tuple => expand_tuple_variant(&variant.fields, TupleVariant::Untagged),
        Style::Newtype => {
          let field: &Field = &variant.fields[0];
          let span: Span = field.original.span();
          let func: TokenStream = quote_spanned!(span=> _diff::Diff::diff);

          quote_expr!(#func(__lhs0, __rhs0, out))
        }
        Style::Unit => quote_expr!(_diff::Differ::same(out, lhs, rhs)),
      },
    });

    quote!(#case => #body)
  }
}

fn expand_ident<T>(prefix: &str, ident: T) -> syn::Ident
where
  T: Display,
{
  syn::Ident::new(&format!("{}{}", prefix, ident), Span::call_site())
}

fn expand_member(field: &Field, mut member: TokenStream) -> TokenStream {
  if let Some(getter) = field.attrs.getter() {
    member = quote!(&#getter(#member));
  }

  if let Some(into) = field.attrs.type_into() {
    member = quote!(&Into::<#into>::into(Clone::clone(#member)));
  }

  member
}

fn expand_struct_fields(fields: &[Field], enum_: bool, trait_: StructTrait) -> Vec<TokenStream> {
  fields
    .iter()
    .filter(|field| !field.attrs.skip_diff())
    .map(|field| {
      let name: String = field.attrs.name().diff_name();
      let span: Span = field.original.span();
      let func: TokenStream = trait_.visit(span);

      let mut lhs: TokenStream;
      let mut rhs: TokenStream;

      let member: &syn::Member = &field.member;

      if enum_ {
        let ident_lhs: syn::Ident = expand_ident("__lhs", quote!(#member));
        let ident_rhs: syn::Ident = expand_ident("__rhs", quote!(#member));

        lhs = quote!(#ident_lhs);
        rhs = quote!(#ident_rhs);
      } else {
        lhs = quote!(&lhs.#member);
        rhs = quote!(&rhs.#member);
      }

      lhs = expand_member(field, lhs);
      rhs = expand_member(field, rhs);

      quote! {
        #func(&mut out, #name, #lhs, #rhs)?;
      }
    })
    .collect()
}

fn expand_tuple_fields(fields: &[Field], enum_: bool, trait_: TupleTrait) -> Vec<TokenStream> {
  fields
    .iter()
    .enumerate()
    .filter(|(_, field)| !field.attrs.skip_diff())
    .map(|(index, field)| {
      let span: Span = field.original.span();
      let func: TokenStream = trait_.visit(span);

      let mut lhs: TokenStream;
      let mut rhs: TokenStream;

      let member: &syn::Member = &field.member;

      if enum_ {
        let ident_lhs: syn::Ident = expand_ident("__lhs", index);
        let ident_rhs: syn::Ident = expand_ident("__rhs", index);

        lhs = quote!(#ident_lhs);
        rhs = quote!(#ident_rhs);
      } else {
        let member2: syn::Member = syn::Member::Unnamed(syn::Index {
          index: index as u32,
          span: Span::call_site(),
        });

        if *member != member2 {
          println!("Member > {}", quote!(#member));
          println!("Generated > {}", quote!(#member2));
          panic!("FIXME");
        }

        lhs = quote!(&lhs.#member);
        rhs = quote!(&rhs.#member);
      }

      lhs = expand_member(field, lhs);
      rhs = expand_member(field, rhs);

      quote! {
        #func(&mut out, #lhs, #rhs)?;
      }
    })
    .collect()
}

fn expand_variant_case(this: &syn::Ident, variant: &Variant) -> TokenStream {
  let ident: &syn::Ident = &variant.ident;

  match variant.style {
    Style::Struct => {
      let lhs_src: _ = variant.fields.iter().map(|field| &field.member);
      let lhs_dst: _ = lhs_src
        .clone()
        .map(|member| expand_ident("__lhs", quote!(#member)));

      let rhs_src: _ = variant.fields.iter().map(|field| &field.member);
      let rhs_dst: _ = rhs_src
        .clone()
        .map(|member| expand_ident("__rhs", quote!(#member)));

      quote! {
        (
          #this::#ident { #(#lhs_src: ref #lhs_dst),* },
          #this::#ident { #(#rhs_src: ref #rhs_dst),* },
        )
      }
    }
    Style::Tuple => {
      let lhs: _ = (0..variant.fields.len()).map(|index| expand_ident("__lhs", index));
      let rhs: _ = (0..variant.fields.len()).map(|index| expand_ident("__rhs", index));

      quote!((#this::#ident(#(ref #lhs),*), #this::#ident(#(ref #rhs),*)))
    }
    Style::Newtype => {
      quote!((#this::#ident(ref __lhs0), #this::#ident(ref __rhs0)))
    }
    Style::Unit => {
      quote!((#this::#ident, #this::#ident))
    }
  }
}

fn expand_struct_variant(fields: &[Field], this: String, context: StructVariant) -> Fragment {
  let trait_: StructTrait = match context {
    StructVariant::External { .. } => StructTrait::DiffStructVariant,
    StructVariant::Untagged => StructTrait::DiffStruct,
  };

  let stmts: Vec<TokenStream> = expand_struct_fields(fields, true, trait_);
  let size: usize = stmts.len();

  match context {
    StructVariant::External { name } => {
      quote_block! {
        let mut out: __D::DiffStructVariant = _diff::Differ::diff_struct_variant(out, #this, #name, #size);
        #(#stmts)*
        _diff::DiffStructVariant::end(out)
      }
    }
    StructVariant::Untagged => {
      quote_block! {
        let mut out: __D::DiffStruct = _diff::Differ::diff_struct(out, #this, #size);
        #(#stmts)*
        _diff::DiffStruct::end(out)
      }
    }
  }
}

fn expand_tuple_variant(fields: &[Field], context: TupleVariant) -> Fragment {
  let trait_: TupleTrait = match context {
    TupleVariant::External { .. } => TupleTrait::DiffTupleVariant,
    TupleVariant::Untagged => TupleTrait::DiffTuple,
  };

  let stmts: Vec<TokenStream> = expand_tuple_fields(fields, true, trait_);
  let size: usize = stmts.len();

  match context {
    TupleVariant::External { this, name } => {
      quote_block! {
        let mut out: __D::DiffTupleVariant = _diff::Differ::diff_tuple_variant(out, #this, #name, #size);
        #(#stmts)*
        _diff::DiffTupleVariant::end(out)
      }
    }
    TupleVariant::Untagged => {
      quote_block! {
        let mut out: __D::DiffTuple = _diff::Differ::diff_tuple(out, #size);
        #(#stmts)*
        _diff::DiffTuple::end(out)
      }
    }
  }
}

enum StructVariant {
  External { name: String },
  Untagged,
}

enum TupleVariant {
  External { this: String, name: String },
  Untagged,
}

enum TupleTrait {
  DiffTuple,
  DiffTupleStruct,
  DiffTupleVariant,
}

impl TupleTrait {
  fn visit(&self, span: Span) -> TokenStream {
    match self {
      Self::DiffTuple => quote_spanned!(span=> _diff::DiffTuple::visit),
      Self::DiffTupleStruct => quote_spanned!(span=> _diff::DiffTupleStruct::visit),
      Self::DiffTupleVariant => quote_spanned!(span=> _diff::DiffTupleVariant::visit),
    }
  }
}

enum StructTrait {
  DiffStruct,
  DiffStructVariant,
}

impl StructTrait {
  fn visit(&self, span: Span) -> TokenStream {
    match self {
      Self::DiffStruct => quote_spanned!(span=> _diff::DiffStruct::visit),
      Self::DiffStructVariant => quote_spanned!(span=> _diff::DiffStructVariant::visit),
    }
  }
}
