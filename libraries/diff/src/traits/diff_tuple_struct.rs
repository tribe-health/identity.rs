use crate::error::Error;
use crate::traits::Diff;

pub trait DiffTupleStruct {
  type Ok;
  type Error: Error;

  fn visit<T>(&mut self, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: Diff + ?Sized;

  fn end(self) -> Result<Self::Ok, Self::Error>;
}
