use proc_macro2::Group;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::ToTokens;
use std::borrow::Cow;
use syn::parse;
use syn::parse::Parse;
use syn::Meta;
use syn::NestedMeta;

use crate::internal::symbol::*;
use crate::internal::Context;

// =============================================================================
// =============================================================================

pub struct Value<'a, T> {
  context: &'a Context,
  name: Symbol,
  tokens: TokenStream,
  value: Option<T>,
}

impl<'a, T> Value<'a, T> {
  pub fn none(context: &'a Context, name: Symbol) -> Self {
    Self {
      context,
      name,
      tokens: TokenStream::new(),
      value: None,
    }
  }

  pub fn get(self) -> Option<T> {
    self.value
  }

  pub fn set<A>(&mut self, object: A, value: T)
  where
    A: ToTokens,
  {
    let tokens: TokenStream = object.into_token_stream();

    if self.value.is_some() {
      self
        .context
        .error_spanned_by(tokens, format!("duplicate patch attribute `{}`", self.name));
    } else {
      self.tokens = tokens;
      self.value = Some(value);
    }
  }

  pub fn set_opt<A>(&mut self, object: A, value: Option<T>)
  where
    A: ToTokens,
  {
    if let Some(value) = value {
      self.set(object, value);
    }
  }
}

// =============================================================================
// =============================================================================

struct BoolValue<'a>(Value<'a, ()>);

impl<'a> BoolValue<'a> {
  fn none(context: &'a Context, name: Symbol) -> Self {
    Self(Value::none(context, name))
  }

  fn set_true<A>(&mut self, object: A)
  where
    A: ToTokens,
  {
    self.0.set(object, ());
  }

  fn get(&self) -> bool {
    self.0.value.is_some()
  }
}

// =============================================================================
// =============================================================================

pub struct Name {
  diff_name: String,
  merge_name: String,
}

impl Name {
  pub fn from_attrs(name: String, diff_name: Value<String>, merge_name: Value<String>) -> Self {
    Self {
      diff_name: diff_name.get().unwrap_or_else(|| name.clone()),
      merge_name: merge_name.get().unwrap_or(name),
    }
  }

  pub fn diff_name(&self) -> String {
    self.diff_name.clone()
  }

  pub fn merge_name(&self) -> String {
    self.merge_name.clone()
  }
}

// =============================================================================
// =============================================================================

struct Customize {
  name: Name,
  getter: Option<syn::ExprPath>,
  skip_diff: bool,
  skip_merge: bool,
  transparent: bool,
  type_from: Option<syn::Type>,
  type_into: Option<syn::Type>,
  type_try_from: Option<syn::Type>,
  untagged: bool,
}

impl Customize {
  pub fn from_ast(
    context: &Context,
    scope: &'static str,
    name: String,
    attrs: &[syn::Attribute],
    input: Option<&syn::DeriveInput>,
    allow: &[Symbol],
  ) -> Self {
    let mut diff_name: Value<String> = Value::none(context, RENAME);
    let mut merge_name: Value<String> = Value::none(context, RENAME);

    let mut getter: Value<syn::ExprPath> = Value::none(context, GETTER);
    let mut skip_diff: BoolValue = BoolValue::none(context, SKIP_DIFF);
    let mut skip_merge: BoolValue = BoolValue::none(context, SKIP_MERGE);
    let mut transparent: BoolValue = BoolValue::none(context, TRANSPARENT);
    let mut type_from: Value<syn::Type> = Value::none(context, FROM);
    let mut type_into: Value<syn::Type> = Value::none(context, INTO);
    let mut type_try_from: Value<syn::Type> = Value::none(context, TRY_FROM);
    let mut untagged: BoolValue = BoolValue::none(context, UNTAGGED);

    let iter: _ = attrs
      .iter()
      .flat_map(|attr| patch_meta_items(context, attr))
      .flatten();

    let include = |word: &syn::Path, name: Symbol| word == name && allow.contains(&name);

    for meta in iter {
      match meta {
        // Parse `#[patch(rename = "foo")]`
        NestedMeta::Meta(Meta::NameValue(meta)) if include(&meta.path, RENAME) => {
          if let Ok(value) = get_lit_str(context, RENAME, &meta.lit) {
            diff_name.set(&meta.path, value.value());
            merge_name.set(&meta.path, value.value());
          }
        }
        // Parse `#[patch(getter = "...")]`
        NestedMeta::Meta(Meta::NameValue(meta)) if include(&meta.path, GETTER) => {
          if let Ok(path) = parse_lit_into_expr_path(context, GETTER, &meta.lit) {
            getter.set(&meta.path, path);
          }
        }
        // Parse `#[patch(skip_diff)]`
        NestedMeta::Meta(Meta::Path(ref word)) if include(word, SKIP_DIFF) => {
          skip_diff.set_true(word);
        }
        // Parse `#[patch(skip_merge)]`
        NestedMeta::Meta(Meta::Path(ref word)) if include(word, SKIP_MERGE) => {
          skip_merge.set_true(word);
        }
        // Parse `#[patch(skip)]`
        NestedMeta::Meta(Meta::Path(ref word)) if include(word, SKIP) => {
          skip_diff.set_true(word);
          skip_merge.set_true(word);
        }
        // Parse `#[patch(from = "Type")]
        NestedMeta::Meta(Meta::NameValue(meta)) if include(&meta.path, FROM) => {
          if let Ok(value) = parse_lit_into_ty(context, FROM, &meta.lit) {
            type_from.set_opt(&meta.path, Some(value));
          }
        }
        // Parse `#[patch(into = "Type")]
        NestedMeta::Meta(Meta::NameValue(meta)) if include(&meta.path, INTO) => {
          if let Ok(value) = parse_lit_into_ty(context, INTO, &meta.lit) {
            type_into.set_opt(&meta.path, Some(value));
          }
        }
        // Parse `#[patch(try_from = "Type")]
        NestedMeta::Meta(Meta::NameValue(meta)) if include(&meta.path, TRY_FROM) => {
          if let Ok(value) = parse_lit_into_ty(context, TRY_FROM, &meta.lit) {
            type_try_from.set_opt(&meta.path, Some(value));
          }
        }
        // Parse `#[patch(transparent)]`
        NestedMeta::Meta(Meta::Path(ref word)) if include(word, TRANSPARENT) => {
          transparent.set_true(word);
        }
        // Parse `#[patch(untagged)]`
        NestedMeta::Meta(Meta::Path(ref word)) if include(word, UNTAGGED) => {
          match input.unwrap().data {
            syn::Data::Enum(_) => {
              untagged.set_true(word);
            }
            syn::Data::Struct(syn::DataStruct { struct_token, .. }) => {
              context
                .error_spanned_by(struct_token, "#[patch(untagged)] can only be used on enums");
            }
            syn::Data::Union(syn::DataUnion { union_token, .. }) => {
              context.error_spanned_by(union_token, "#[patch(untagged)] can only be used on enums");
            }
          }
        }
        NestedMeta::Meta(item) => {
          let path: String = item.path().into_token_stream().to_string().replace(' ', "");

          context.error_spanned_by(
            item.path(),
            format!("unknown patch {} attribute `{}`", scope, path),
          );
        }
        NestedMeta::Lit(lit) => {
          context.error_spanned_by(
            lit,
            format!("unexpected literal in patch {} attribute", scope),
          );
        }
      }
    }

    Self {
      name: Name::from_attrs(name, diff_name, merge_name),
      getter: getter.get(),
      skip_diff: skip_diff.get(),
      skip_merge: skip_merge.get(),
      transparent: transparent.get(),
      type_from: type_from.get(),
      type_into: type_into.get(),
      type_try_from: type_try_from.get(),
      untagged: untagged.get(),
    }
  }

  pub fn name(&self) -> &Name {
    &self.name
  }

  pub fn getter(&self) -> Option<&syn::ExprPath> {
    self.getter.as_ref()
  }

  pub fn skip_diff(&self) -> bool {
    self.skip_diff
  }

  pub fn skip_merge(&self) -> bool {
    self.skip_merge
  }

  pub fn transparent(&self) -> bool {
    self.transparent
  }

  pub fn type_from(&self) -> Option<&syn::Type> {
    self.type_from.as_ref()
  }

  pub fn type_into(&self) -> Option<&syn::Type> {
    self.type_into.as_ref()
  }

  pub fn type_try_from(&self) -> Option<&syn::Type> {
    self.type_try_from.as_ref()
  }

  pub fn untagged(&self) -> bool {
    self.untagged
  }
}

// =============================================================================
// =============================================================================

pub struct Field {
  attrs: Customize,
  transparent: bool,
}

impl Field {
  pub fn from_ast(context: &Context, index: usize, item: &syn::Field) -> Self {
    let name: String = item
      .ident
      .as_ref()
      .map(unraw)
      .unwrap_or_else(|| index.to_string());

    let list: &[Symbol] = &[
      FROM, GETTER, INTO, RENAME, SKIP, SKIP_DIFF, SKIP_MERGE, TRY_FROM,
    ];

    let attrs: Customize = Customize::from_ast(context, "field", name, &item.attrs, None, list);

    Self {
      attrs,
      transparent: false,
    }
  }

  pub fn flag_transparent(&mut self) {
    self.transparent = true;
  }

  pub fn transparent(&self) -> bool {
    self.transparent
  }

  pub fn name(&self) -> &Name {
    self.attrs.name()
  }

  pub fn getter(&self) -> Option<&syn::ExprPath> {
    self.attrs.getter()
  }

  pub fn skip_diff(&self) -> bool {
    self.attrs.skip_diff()
  }

  pub fn skip_merge(&self) -> bool {
    self.attrs.skip_merge()
  }

  pub fn type_from(&self) -> Option<&syn::Type> {
    self.attrs.type_from()
  }

  pub fn type_into(&self) -> Option<&syn::Type> {
    self.attrs.type_into()
  }

  pub fn type_try_from(&self) -> Option<&syn::Type> {
    self.attrs.type_try_from()
  }
}

// =============================================================================
// =============================================================================

pub struct Variant {
  attrs: Customize,
}

impl Variant {
  pub fn from_ast(context: &Context, item: &syn::Variant) -> Self {
    let name: String = unraw(&item.ident);
    let list: &[Symbol] = &[RENAME, SKIP, SKIP_DIFF, SKIP_MERGE];
    let attrs: Customize = Customize::from_ast(context, "variant", name, &item.attrs, None, list);

    Self { attrs }
  }

  pub fn name(&self) -> &Name {
    &self.attrs.name()
  }

  pub fn skip_diff(&self) -> bool {
    self.attrs.skip_diff()
  }

  pub fn skip_merge(&self) -> bool {
    self.attrs.skip_merge()
  }
}

// =============================================================================
// =============================================================================

#[derive(Clone, Copy)]
pub enum TagType {
  External,
  Untagged,
}

impl TagType {
  pub const fn new(untagged: bool) -> Self {
    if untagged {
      Self::Untagged
    } else {
      Self::External
    }
  }
}

// =============================================================================
// =============================================================================

pub struct Attributes {
  tag: TagType,
  attrs: Customize,
}

impl Attributes {
  pub fn from_ast(context: &Context, item: &syn::DeriveInput) -> Self {
    let name: String = unraw(&item.ident);
    let list: &[Symbol] = &[FROM, INTO, RENAME, TRANSPARENT, TRY_FROM, UNTAGGED];
    let attrs: Customize =
      Customize::from_ast(context, "container", name, &item.attrs, Some(item), list);
    let tag: TagType = TagType::new(attrs.untagged());

    Self { tag, attrs }
  }

  pub fn tag(&self) -> TagType {
    self.tag
  }

  pub fn name(&self) -> &Name {
    self.attrs.name()
  }

  pub fn transparent(&self) -> bool {
    self.attrs.transparent()
  }

  pub fn type_from(&self) -> Option<&syn::Type> {
    self.attrs.type_from()
  }

  pub fn type_into(&self) -> Option<&syn::Type> {
    self.attrs.type_into()
  }

  pub fn type_try_from(&self) -> Option<&syn::Type> {
    self.attrs.type_try_from()
  }

  // TODO: Support custom paths (?)
  pub fn __lib(&self) -> Cow<syn::Path> {
    Cow::Owned(parse_quote!(_diff))
  }
}

// =============================================================================
// =============================================================================

fn unraw(ident: &syn::Ident) -> String {
  ident.to_string().trim_start_matches("r#").to_owned()
}

fn patch_meta_items(context: &Context, attr: &syn::Attribute) -> Result<Vec<NestedMeta>, ()> {
  if attr.path != PATCH {
    return Ok(Vec::new());
  }

  match attr.parse_meta() {
    Ok(syn::Meta::List(meta)) => Ok(meta.nested.into_iter().collect()),
    Ok(other) => {
      context.error_spanned_by(other, "expected #[patch(...)]");
      Err(())
    }
    Err(error) => {
      context.syn_error(error);
      Err(())
    }
  }
}

fn get_lit_str<'a>(
  context: &Context,
  attr_name: Symbol,
  lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
  get_lit_str2(context, attr_name, attr_name, lit)
}

fn get_lit_str2<'a>(
  context: &Context,
  name: Symbol,
  item: Symbol,
  lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
  if let syn::Lit::Str(lit) = lit {
    Ok(lit)
  } else {
    context.error_spanned_by(
      lit,
      format!(
        "expected patch {} attribute to be a string: `{} = \"...\"`",
        name, item
      ),
    );
    Err(())
  }
}

fn parse_lit_into_ty(context: &Context, name: Symbol, lit: &syn::Lit) -> Result<syn::Type, ()> {
  let string: &syn::LitStr = get_lit_str(context, name, lit)?;

  parse_lit_str(string).map_err(|_| {
    context.error_spanned_by(
      lit,
      format!("failed to parse type: {} = {:?}", name, string.value()),
    )
  })
}

fn parse_lit_into_expr_path(
  context: &Context,
  name: Symbol,
  lit: &syn::Lit,
) -> Result<syn::ExprPath, ()> {
  let string: &syn::LitStr = get_lit_str(context, name, lit)?;

  parse_lit_str(string).map_err(|_| {
    context.error_spanned_by(lit, format!("failed to parse path: {:?}", string.value()))
  })
}

fn parse_lit_str<T>(string: &syn::LitStr) -> parse::Result<T>
where
  T: Parse,
{
  syn::parse2(spanned_tokens(string)?)
}

fn spanned_tokens(string: &syn::LitStr) -> parse::Result<TokenStream> {
  let stream = syn::parse_str(&string.value())?;
  Ok(respan_token_stream(stream, string.span()))
}

fn respan_token_stream(stream: TokenStream, span: Span) -> TokenStream {
  stream
    .into_iter()
    .map(|token| respan_token_tree(token, span))
    .collect()
}

fn respan_token_tree(mut token: TokenTree, span: Span) -> TokenTree {
  if let TokenTree::Group(ref mut group) = token {
    *group = Group::new(group.delimiter(), respan_token_stream(group.stream(), span));
  }

  token.set_span(span);
  token
}
