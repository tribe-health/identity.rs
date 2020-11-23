use core::cmp::Ordering;
use core::iter::Fuse;

use crate::utils::put_back;
use crate::utils::EoB;
use crate::utils::PutBack;

#[inline]
pub fn merge_join_by<I, J, F>(a: I, b: J, f: F) -> MergeJoinBy<I::IntoIter, J::IntoIter, F>
where
  I: IntoIterator,
  J: IntoIterator,
  F: FnMut(&I::Item, &J::Item) -> Ordering,
{
  MergeJoinBy {
    a: put_back(a.into_iter().fuse()),
    b: put_back(b.into_iter().fuse()),
    f,
  }
}

pub struct MergeJoinBy<I, J, F>
where
  I: Iterator,
  J: Iterator,
{
  a: PutBack<Fuse<I>>,
  b: PutBack<Fuse<J>>,
  f: F,
}

impl<I, J, F> Iterator for MergeJoinBy<I, J, F>
where
  I: Iterator,
  J: Iterator,
  F: FnMut(&I::Item, &J::Item) -> Ordering,
{
  type Item = EoB<I::Item, J::Item>;

  fn next(&mut self) -> Option<Self::Item> {
    match (self.a.next(), self.b.next()) {
      (None, None) => None,
      (Some(a), None) => Some(EoB::Lhs(a)),
      (None, Some(b)) => Some(EoB::Rhs(b)),
      (Some(a), Some(b)) => match (self.f)(&a, &b) {
        Ordering::Equal => Some(EoB::Both(a, b)),
        Ordering::Less => {
          self.b.put_back(b);
          Some(EoB::Lhs(a))
        }
        Ordering::Greater => {
          self.a.put_back(a);
          Some(EoB::Rhs(b))
        }
      },
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let (al, au): (usize, Option<usize>) = self.a.size_hint();
    let (bl, bu): (usize, Option<usize>) = self.b.size_hint();

    (al.max(bl), au.zip(bu).and_then(|(a, b)| a.checked_add(b)))
  }
}
