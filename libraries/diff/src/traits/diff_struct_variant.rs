use crate::error::Error;
use crate::traits::Diff;

pub trait DiffStructVariant {
  type Ok;
  type Error: Error;

  fn visit<T>(&mut self, key: &'static str, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: Diff + ?Sized;

  fn end(self) -> Result<Self::Ok, Self::Error>;
}
