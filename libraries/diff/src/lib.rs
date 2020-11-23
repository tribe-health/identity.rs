#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

// Re-export `serde` for convenience.
pub use serde;

mod error;
mod impls;
mod traits;
mod utils;

#[cfg(all(feature = "serde_json", feature = "itoa", feature = "std"))]
pub mod json;

pub use self::error::Error;
pub use self::traits::Diff;
pub use self::traits::DiffMap;
pub use self::traits::DiffSeq;
pub use self::traits::DiffStruct;
pub use self::traits::DiffStructVariant;
pub use self::traits::DiffTuple;
pub use self::traits::DiffTupleStruct;
pub use self::traits::DiffTupleVariant;
pub use self::traits::Differ;

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate internal;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use internal::*;

mod lib {
  pub use core::cell::{Cell, RefCell};
  pub use core::cmp::Reverse;
  pub use core::fmt::{Debug, Display};
  pub use core::marker::PhantomData;
  pub use core::num::Wrapping;
  pub use core::num::{NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize};
  pub use core::num::{NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize};
  pub use core::ops::{Range, RangeInclusive};
  pub use core::time::Duration;

  #[cfg(all(feature = "alloc", not(feature = "std")))]
  pub use alloc::borrow::{Cow, ToOwned};
  #[cfg(feature = "std")]
  pub use std::borrow::{Cow, ToOwned};

  #[cfg(all(feature = "alloc", not(feature = "std")))]
  pub use alloc::string::String;
  #[cfg(feature = "std")]
  pub use std::string::String;

  #[cfg(all(feature = "alloc", not(feature = "std")))]
  pub use alloc::vec::Vec;
  #[cfg(feature = "std")]
  pub use std::vec::Vec;

  #[cfg(all(feature = "alloc", not(feature = "std")))]
  pub use alloc::boxed::Box;
  #[cfg(feature = "std")]
  pub use std::boxed::Box;

  #[cfg(all(feature = "alloc", not(feature = "std")))]
  pub use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
  #[cfg(feature = "std")]
  pub use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

  #[cfg(feature = "std")]
  pub use std::collections::{HashMap, HashSet};
  #[cfg(feature = "std")]
  pub use std::hash::{BuildHasher, Hash};
  #[cfg(feature = "std")]
  pub use std::path::{Path, PathBuf};
  #[cfg(feature = "std")]
  pub use std::sync::{Mutex, RwLock};
}
