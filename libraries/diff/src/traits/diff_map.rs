use core::fmt::Display;

use crate::error::Error;
use crate::traits::Diff;

pub trait DiffMap {
  type Ok;
  type Error: Error;

  fn visit<T, U>(&mut self, key: &T, lhs: &U, rhs: &U) -> Result<(), Self::Error>
  where
    T: Display + ?Sized,
    U: Diff + ?Sized;

  fn visit_lhs<T, U>(&mut self, key: &T, lhs: &U) -> Result<(), Self::Error>
  where
    T: Display + ?Sized,
    U: Diff + ?Sized;

  fn visit_rhs<T, U>(&mut self, key: &T, rhs: &U) -> Result<(), Self::Error>
  where
    T: Display + ?Sized,
    U: Diff + ?Sized;

  fn end(self) -> Result<Self::Ok, Self::Error>;
}
