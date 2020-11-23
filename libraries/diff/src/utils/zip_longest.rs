use core::iter::Fuse;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum EoB<T, U> {
  Both(T, U),
  Lhs(T),
  Rhs(U),
}

pub struct ZipLongest<T, U> {
  a: Fuse<T>,
  b: Fuse<U>,
}

#[inline]
pub fn zip_longest<T, U>(a: T, b: U) -> ZipLongest<T, U>
where
  T: Iterator,
  U: Iterator,
{
  ZipLongest {
    a: a.fuse(),
    b: b.fuse(),
  }
}

impl<T, U> Iterator for ZipLongest<T, U>
where
  T: Iterator,
  U: Iterator,
{
  type Item = EoB<T::Item, U::Item>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    match (self.a.next(), self.b.next()) {
      (Some(a), Some(b)) => Some(EoB::Both(a, b)),
      (Some(a), None) => Some(EoB::Lhs(a)),
      (None, Some(b)) => Some(EoB::Rhs(b)),
      (None, None) => None,
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let (al, au): (usize, Option<usize>) = self.a.size_hint();
    let (bl, bu): (usize, Option<usize>) = self.b.size_hint();

    (al.max(bl), au.zip(bu).map(|(a, b)| a.max(b)))
  }
}
