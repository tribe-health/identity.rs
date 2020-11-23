#[inline]
pub fn put_back<I>(iter: I) -> PutBack<I::IntoIter>
where
  I: IntoIterator,
{
  PutBack {
    top: None,
    iter: iter.into_iter(),
  }
}

pub struct PutBack<I>
where
  I: Iterator,
{
  top: Option<I::Item>,
  iter: I,
}

impl<I> PutBack<I>
where
  I: Iterator,
{
  #[inline]
  pub fn put_back(&mut self, item: I::Item) {
    self.top = Some(item)
  }
}

impl<I> Iterator for PutBack<I>
where
  I: Iterator,
{
  type Item = I::Item;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    match self.top {
      None => self.iter.next(),
      ref mut some => some.take(),
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let (mut lo, mut hi) = self.iter.size_hint();

    lo = lo.saturating_add(self.top.is_some() as usize);
    hi = hi.and_then(|hi| hi.checked_add(self.top.is_some() as usize));

    (lo, hi)
  }
}
