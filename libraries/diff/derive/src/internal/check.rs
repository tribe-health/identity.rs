use crate::internal::ungroup;
use crate::internal::Container;
use crate::internal::Context;
use crate::internal::Data;
use crate::internal::Derive;
use crate::internal::Field;
use crate::internal::Style;

/// Cross-cutting checks that require looking at more than a single attrs
/// object. Simpler checks should happen when parsing and building the attrs.
pub fn check(context: &Context, container: &mut Container, derive: Derive) {
  check_transparent(context, container, derive);
  check_from_and_try_from(context, container);
}

/// Enums and unit structs cannot be transparent.
fn check_transparent(context: &Context, container: &mut Container, derive: Derive) {
  if !container.attrs.transparent() {
    return;
  }

  if container.attrs.type_from().is_some() {
    context.error_spanned_by(
      container.original,
      "#[patch(transparent)] is not allowed with #[patch(from = \"...\")]",
    );
  }

  if container.attrs.type_try_from().is_some() {
    context.error_spanned_by(
      container.original,
      "#[patch(transparent)] is not allowed with #[patch(try_from = \"...\")]",
    );
  }

  if container.attrs.type_into().is_some() {
    context.error_spanned_by(
      container.original,
      "#[patch(transparent)] is not allowed with #[patch(into = \"...\")]",
    );
  }

  let fields: &mut [Field] = match container.data {
    Data::Enum(_) => {
      context.error_spanned_by(
        container.original,
        "#[patch(transparent)] is not allowed on an enum",
      );
      return;
    }
    Data::Struct(Style::Unit, _) => {
      context.error_spanned_by(
        container.original,
        "#[patch(transparent)] is not allowed on a unit struct",
      );
      return;
    }
    Data::Struct(_, ref mut fields) => fields,
  };

  let mut target: Option<&mut Field> = None;

  for field in fields {
    if allow_transparent(field, derive) {
      if target.is_some() {
        context.error_spanned_by(
          container.original,
          "#[patch(transparent)] requires struct to have at most one transparent field",
        );
        return;
      }
      target = Some(field);
    }
  }

  match target {
    Some(target) => {
      target.attrs.flag_transparent();
    }
    None => match derive {
      Derive::Diff => {
        context.error_spanned_by(
          container.original,
          "#[patch(transparent)] requires at least one field that is not skipped",
        );
      }
    },
  }
}

fn allow_transparent(field: &Field, derive: Derive) -> bool {
  if let syn::Type::Path(ty) = ungroup(&field.ty) {
    if let Some(segment) = ty.path.segments.last() {
      if segment.ident == "PhantomData" {
        return false;
      }
    }
  }

  match derive {
    Derive::Diff => !field.attrs.skip_diff(),
  }
}

fn check_from_and_try_from(context: &Context, container: &mut Container) {
  if container.attrs.type_from().is_some() && container.attrs.type_try_from().is_some() {
    context.error_spanned_by(
      container.original,
      "#[patch(from = \"...\")] and #[patch(try_from = \"...\")] conflict with each other",
    )
  }
}
