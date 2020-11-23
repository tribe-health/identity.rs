use crate::error::Error;
use crate::traits::Diff;
use crate::utils::zip_longest;
use crate::utils::EoB;

pub trait DiffSeq {
  type Ok;
  type Error: Error;

  fn visit<T>(&mut self, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: Diff + ?Sized;

  fn visit_lhs<T>(&mut self, lhs: &T) -> Result<(), Self::Error>
  where
    T: Diff + ?Sized;

  fn visit_rhs<T>(&mut self, rhs: &T) -> Result<(), Self::Error>
  where
    T: Diff + ?Sized;

  fn end(self) -> Result<Self::Ok, Self::Error>;

  #[inline]
  fn visit_all<I>(&mut self, lhs: I, rhs: I) -> Result<(), Self::Error>
  where
    I: IntoIterator,
    <I as IntoIterator>::Item: Diff,
  {
    for zipzap in zip_longest(lhs.into_iter(), rhs.into_iter()) {
      match zipzap {
        EoB::Both(ref lhs, ref rhs) => self.visit(lhs, rhs)?,
        EoB::Lhs(ref lhs) => self.visit_lhs(lhs)?,
        EoB::Rhs(ref rhs) => self.visit_rhs(rhs)?,
      }
    }

    Ok(())
  }
}
