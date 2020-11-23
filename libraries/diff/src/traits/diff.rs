use serde::Serialize;

use crate::traits::Differ;

pub trait Diff: Serialize {
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ;
}
