mod attributes;
mod check;
mod container;
mod context;
pub mod symbol;

pub use self::attributes::Attributes;
pub use self::attributes::Field as FieldAttrs;
pub use self::attributes::Name;
pub use self::attributes::TagType;
pub use self::attributes::Value;
pub use self::attributes::Variant as VariantAttrs;

pub use self::container::Container;
pub use self::container::Data;
pub use self::container::Field;
pub use self::container::Style;
pub use self::container::Variant;
pub use self::context::Context;

#[derive(Clone, Copy)]
pub enum Derive {
  Diff,
}

pub fn ungroup(mut ty: &syn::Type) -> &syn::Type {
  while let syn::Type::Group(group) = ty {
    ty = &group.elem;
  }

  ty
}
