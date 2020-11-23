use crate::error::Error as _;
use crate::lib::*;
use crate::traits::*;
use crate::utils::*;

// =============================================================================

macro_rules! impl_cmp {
  ($($ty:ty)+) => {
    $(
      impl Diff for $ty {
        fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error> where D: Differ {
          if lhs == rhs {
            out.same(lhs, rhs)
          } else {
            out.difference(lhs, rhs)
          }
        }
      }
    )+
  };
}

// =============================================================================

impl_cmp!(bool);
impl_cmp!(char);
impl_cmp!(i8 i16 i32 i64 isize i128);
impl_cmp!(u8 u16 u32 u64 usize u128);
impl_cmp!(f32 f64);

// =============================================================================

impl_cmp!(str);
#[cfg(any(feature = "std", feature = "alloc"))]
impl_cmp!(String);

// =============================================================================

impl<T> Diff for Option<T>
where
  T: Diff,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    match (lhs, rhs) {
      (Some(lhs), Some(rhs)) => Diff::diff(lhs, rhs, out),
      (None, None) => out.same(lhs, rhs),
      (_, _) => out.difference(lhs, rhs),
    }
  }
}

// =============================================================================

impl<T> Diff for PhantomData<T>
where
  T: ?Sized,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    out.same(lhs, rhs)
  }
}

// =============================================================================

impl<T> Diff for [T; 0] {
  #[inline]
  fn diff<D>(_lhs: &Self, _rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    out.diff_tuple(0).end()
  }
}

macro_rules! impl_diff_array {
  ($($size:expr)+) => {
    $(
      impl<T> Diff for [T; $size] where T: Diff {
        #[inline]
        fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error> where D: Differ {
          let mut out: D::DiffTuple = out.diff_tuple($size);
          for (lhs, rhs) in lhs.iter().zip(rhs.iter()) {
            out.visit(lhs, rhs)?;
          }
          out.end()
        }
      }
    )+
  };
}

impl_diff_array! {
  01 02 03 04 05 06 07 08 09 10
  11 12 13 14 15 16 17 18 19 20
  21 22 23 24 25 26 27 28 29 30
  31 32
}

// =============================================================================

impl<T> Diff for [T]
where
  T: Diff,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffSeq = out.diff_seq(Some(lhs.len().max(rhs.len())));
    out.visit_all(lhs, rhs)?;
    out.end()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Diff for Vec<T>
where
  T: Diff,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    Diff::diff(&**lhs, &**rhs, out)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Diff for VecDeque<T>
where
  T: Diff,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffSeq = out.diff_seq(Some(lhs.len().max(rhs.len())));
    out.visit_all(lhs, rhs)?;
    out.end()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Diff for BinaryHeap<T>
where
  T: Diff + Ord,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffSeq = out.diff_seq(Some(lhs.len().max(rhs.len())));
    out.visit_all(lhs, rhs)?;
    out.end()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Diff for LinkedList<T>
where
  T: Diff,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffSeq = out.diff_seq(Some(lhs.len().max(rhs.len())));
    out.visit_all(lhs, rhs)?;
    out.end()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Diff for BTreeSet<T>
where
  T: Ord + Diff,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffSeq = out.diff_seq(Some(lhs.len().max(rhs.len())));

    for zipzap in merge_join_by(lhs, rhs, |lhs, rhs| lhs.cmp(rhs)) {
      match zipzap {
        EoB::Both(lhs, rhs) => out.visit(lhs, rhs)?,
        EoB::Lhs(lhs) => out.visit_lhs(lhs)?,
        EoB::Rhs(rhs) => out.visit_rhs(rhs)?,
      }
    }

    out.end()
  }
}

#[cfg(feature = "std")]
impl<T, U> Diff for HashSet<T, U>
where
  T: Eq + Hash + Diff,
  U: BuildHasher,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffSeq = out.diff_seq(Some(lhs.len().max(rhs.len())));

    for item in lhs.intersection(&rhs) {
      out.visit(item, rhs.get(item).unwrap())?;
    }

    for item in rhs.difference(&lhs) {
      out.visit_rhs(item)?;
    }

    for item in lhs.difference(&rhs) {
      out.visit_lhs(item)?;
    }

    out.end()
  }
}

// =============================================================================

impl<T> Diff for Range<T>
where
  T: Diff,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffStruct = out.diff_struct("Range", 2);
    out.visit("start", &lhs.start, &rhs.start)?;
    out.visit("end", &lhs.end, &rhs.end)?;
    out.end()
  }
}

// =============================================================================

impl<T> Diff for RangeInclusive<T>
where
  T: Diff,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffStruct = out.diff_struct("RangeInclusive", 2);
    out.visit("start", &lhs.start(), &rhs.start())?;
    out.visit("end", &lhs.end(), &rhs.end())?;
    out.end()
  }
}

// =============================================================================

impl Diff for () {
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    out.same(lhs, rhs)
  }
}

// =============================================================================

macro_rules! impl_diff_tuple {
  ($($size:expr => ($($index:tt $name:ident)+))+) => {
    $(
      impl<$($name),+> Diff for ($($name,)+) where $($name: Diff,)+ {
        #[inline]
        fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error> where D: Differ {
          let mut out: D::DiffTuple = out.diff_tuple($size);
          $(
            out.visit(&lhs.$index, &rhs.$index)?;
          )+
          out.end()
        }
      }
    )+
  };
}

impl_diff_tuple! {
  1 => (0 T0)
  2 => (0 T0 1 T1)
  3 => (0 T0 1 T1 2 T2)
  4 => (0 T0 1 T1 2 T2 3 T3)
  5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
  6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
  7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
  8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
  9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
  10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
  11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
  12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
  13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
  14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
  15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
  16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

// =============================================================================

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T, U> Diff for BTreeMap<T, U>
where
  T: Ord + Display + serde::Serialize,
  U: Diff,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffMap = out.diff_map();

    for zipzap in merge_join_by(lhs, rhs, |(lhs, _), (rhs, _)| lhs.cmp(rhs)) {
      match zipzap {
        EoB::Both((key, lhs), (_, rhs)) => out.visit(key, lhs, rhs)?,
        EoB::Lhs((key, value)) => out.visit_lhs(key, value)?,
        EoB::Rhs((key, value)) => out.visit_rhs(key, value)?,
      }
    }

    out.end()
  }
}

#[cfg(feature = "std")]
impl<T, U, V> Diff for HashMap<T, U, V>
where
  T: Eq + Hash + Display + serde::Serialize,
  U: Diff,
  V: BuildHasher,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffMap = out.diff_map();

    for (key, lhs) in lhs {
      if let Some(rhs) = rhs.get(key) {
        out.visit(key, lhs, rhs)?;
      } else {
        out.visit_lhs(key, lhs)?;
      }
    }

    for (key, rhs) in rhs {
      if !lhs.contains_key(key) {
        out.visit_rhs(key, rhs)?;
      }
    }

    out.end()
  }
}

// =============================================================================

macro_rules! impl_deref {
  ($($tt:tt)+) => {
    impl $($tt)+ {
      #[inline]
      fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
      where
        D: Differ,
      {
        Diff::diff(&**lhs, &**rhs, out)
      }
    }
  };
}

impl_deref!(<T> Diff for &T where T: Diff + ?Sized);
impl_deref!(<T> Diff for &mut T where T: Diff + ?Sized);

#[cfg(any(feature = "std", feature = "alloc"))]
impl_deref!(<T> Diff for Box<T> where T: Diff + ?Sized);

#[cfg(any(feature = "std", feature = "alloc"))]
impl_deref!(<'a, T> Diff for Cow<'a, T> where T: Diff + ToOwned + ?Sized);

// =============================================================================

macro_rules! impl_nonzero {
  ($($ident:ident)+) => {
    $(
      impl Diff for $ident {
        fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error> where D: Differ {
          Diff::diff(&lhs.get(), &rhs.get(), out)
        }
      }
    )+
  };
}

impl_nonzero! {
  NonZeroU8
  NonZeroU16
  NonZeroU32
  NonZeroU64
  NonZeroUsize
}

impl_nonzero! {
  NonZeroI8
  NonZeroI16
  NonZeroI32
  NonZeroI64
  NonZeroIsize
}

// =============================================================================

impl<T> Diff for Cell<T>
where
  T: Diff + Copy,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    Diff::diff(&lhs.get(), &rhs.get(), out)
  }
}

impl<T> Diff for RefCell<T>
where
  T: Diff,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    match (lhs.try_borrow(), rhs.try_borrow()) {
      (Ok(lhs), Ok(rhs)) => Diff::diff(&*lhs, &*rhs, out),
      (Ok(_), Err(_)) => Err(D::Error::custom("refcell mutably borrowed (lhs)")),
      (Err(_), Ok(_)) => Err(D::Error::custom("refcell mutably borrowed (rhs)")),
      (Err(_), Err(_)) => Err(D::Error::custom("refcell mutably borrowed (lhs & rhs)")),
    }
  }
}

#[cfg(feature = "std")]
impl<T> Diff for Mutex<T>
where
  T: Diff,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    match (lhs.lock(), rhs.lock()) {
      (Ok(lhs), Ok(rhs)) => Diff::diff(&*lhs, &*rhs, out),
      (Ok(_), Err(_)) => Err(D::Error::custom("mutex poisoned (lhs)")),
      (Err(_), Ok(_)) => Err(D::Error::custom("mutex poisoned (rhs)")),
      (Err(_), Err(_)) => Err(D::Error::custom("mutex poisoned (lhs & rhs)")),
    }
  }
}

#[cfg(feature = "std")]
impl<T> Diff for RwLock<T>
where
  T: Diff,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    match (lhs.read(), rhs.read()) {
      (Ok(lhs), Ok(rhs)) => Diff::diff(&*lhs, &*rhs, out),
      (Ok(_), Err(_)) => Err(D::Error::custom("rwlock poisoned (lhs)")),
      (Err(_), Ok(_)) => Err(D::Error::custom("rwlock poisoned (rhs)")),
      (Err(_), Err(_)) => Err(D::Error::custom("rwlock poisoned (lhs & rhs)")),
    }
  }
}

// =============================================================================

impl<T, E> Diff for Result<T, E>
where
  T: Diff,
  E: Diff,
{
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    match (lhs, rhs) {
      (Ok(lhs), Ok(rhs)) => out.diff_newtype_variant("Result", "Ok", lhs, rhs),
      (Err(lhs), Err(rhs)) => out.diff_newtype_variant("Result", "Err", lhs, rhs),
      (_, _) => out.difference(lhs, rhs),
    }
  }
}

// =============================================================================

impl Diff for Duration {
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffStruct = out.diff_struct("Duration", 2);
    out.visit("secs", &lhs.as_secs(), &rhs.as_secs())?;
    out.visit("nanos", &lhs.subsec_nanos(), &rhs.subsec_nanos())?;
    out.end()
  }
}

// =============================================================================

#[cfg(feature = "std")]
impl Diff for Path {
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    match (lhs.to_str(), rhs.to_str()) {
      (Some(lhs), Some(rhs)) => Diff::diff(&*lhs, &*rhs, out),
      (Some(_), None) => Err(D::Error::custom(
        "path contains invalid UTF-8 characters (lhs)",
      )),
      (None, Some(_)) => Err(D::Error::custom(
        "path contains invalid UTF-8 characters (rhs)",
      )),
      (None, None) => Err(D::Error::custom(
        "path contains invalid UTF-8 characters (lhs & rhs)",
      )),
    }
  }
}

#[cfg(feature = "std")]
impl Diff for PathBuf {
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    Diff::diff(lhs.as_path(), rhs.as_path(), out)
  }
}

// =============================================================================

#[cfg(feature = "std")]
impl<T> Diff for Wrapping<T>
where
  T: Diff,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    Diff::diff(&lhs.0, &rhs.0, out)
  }
}

impl<T> Diff for Reverse<T>
where
  T: Diff,
{
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    Diff::diff(&lhs.0, &rhs.0, out)
  }
}

// =============================================================================

#[cfg(feature = "did_url")]
impl Diff for did_url::DID {
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    Diff::diff(lhs.as_str(), rhs.as_str(), out)
  }
}

// =============================================================================

#[cfg(all(feature = "serde_json", any(feature = "alloc", feature = "std")))]
impl Diff for serde_json::Map<String, serde_json::Value> {
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    let mut out: D::DiffMap = out.diff_map();

    for (key, lhs) in lhs {
      if let Some(rhs) = rhs.get(key) {
        out.visit(key, lhs, rhs)?;
      } else {
        out.visit_lhs(key, lhs)?;
      }
    }

    for (key, rhs) in rhs {
      if !lhs.contains_key(key) {
        out.visit_rhs(key, rhs)?;
      }
    }

    out.end()
  }
}

#[cfg(all(feature = "serde_json", any(feature = "alloc", feature = "std")))]
impl Diff for serde_json::Number {
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    if let Some((ref lhs, ref rhs)) = lhs.as_u64().zip(rhs.as_u64()) {
      Diff::diff(lhs, rhs, out)
    } else if let Some((ref lhs, ref rhs)) = lhs.as_i64().zip(rhs.as_i64()) {
      Diff::diff(lhs, rhs, out)
    } else if let Some((ref lhs, ref rhs)) = lhs.as_f64().zip(rhs.as_f64()) {
      Diff::diff(lhs, rhs, out)
    } else {
      out.difference(lhs, rhs)
    }
  }
}

#[cfg(all(feature = "serde_json", any(feature = "alloc", feature = "std")))]
impl Diff for serde_json::Value {
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    use serde_json::Value::*;
    match (lhs, rhs) {
      (Null, Null) => Diff::diff(&(), &(), out),
      (Bool(lhs), Bool(rhs)) => Diff::diff(lhs, rhs, out),
      (Number(lhs), Number(rhs)) => Diff::diff(lhs, rhs, out),
      (String(lhs), String(rhs)) => Diff::diff(&**lhs, &**rhs, out),
      (Array(lhs), Array(rhs)) => Diff::diff(&**lhs, &**rhs, out),
      (Object(lhs), Object(rhs)) => Diff::diff(lhs, rhs, out),
      (lhs, rhs) => out.difference(lhs, rhs),
    }
  }
}

// =============================================================================

#[cfg(feature = "url")]
impl Diff for url::Url {
  #[inline]
  fn diff<D>(lhs: &Self, rhs: &Self, out: D) -> Result<D::Ok, D::Error>
  where
    D: Differ,
  {
    Diff::diff(lhs.as_str(), rhs.as_str(), out)
  }
}
