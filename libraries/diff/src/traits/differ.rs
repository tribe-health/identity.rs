use serde::Serialize;

use crate::error::Error;
use crate::traits::Diff;
use crate::traits::DiffMap;
use crate::traits::DiffSeq;
use crate::traits::DiffStruct;
use crate::traits::DiffStructVariant;
use crate::traits::DiffTuple;
use crate::traits::DiffTupleStruct;
use crate::traits::DiffTupleVariant;

pub trait Differ: Sized {
  type Ok;
  type Error: Error;

  type DiffMap: DiffMap<Ok = Self::Ok, Error = Self::Error>;
  type DiffSeq: DiffSeq<Ok = Self::Ok, Error = Self::Error>;
  type DiffStruct: DiffStruct<Ok = Self::Ok, Error = Self::Error>;
  type DiffStructVariant: DiffStructVariant<Ok = Self::Ok, Error = Self::Error>;
  type DiffTuple: DiffTuple<Ok = Self::Ok, Error = Self::Error>;
  type DiffTupleStruct: DiffTupleStruct<Ok = Self::Ok, Error = Self::Error>;
  type DiffTupleVariant: DiffTupleVariant<Ok = Self::Ok, Error = Self::Error>;

  fn difference<T>(self, lhs: &T, rhs: &T) -> Result<Self::Ok, Self::Error>
  where
    T: Serialize + ?Sized;

  fn same<T>(self, lhs: &T, rhs: &T) -> Result<Self::Ok, Self::Error>
  where
    T: Serialize + ?Sized;

  fn diff_map(self) -> Self::DiffMap;

  fn diff_seq(self, size: Option<usize>) -> Self::DiffSeq;

  fn diff_struct(self, name: &'static str, size: usize) -> Self::DiffStruct;

  fn diff_struct_variant(
    self,
    name: &'static str,
    variant: &'static str,
    size: usize,
  ) -> Self::DiffStructVariant;

  fn diff_tuple(self, size: usize) -> Self::DiffTuple;

  fn diff_tuple_struct(self, name: &'static str, size: usize) -> Self::DiffTupleStruct;

  fn diff_tuple_variant(
    self,
    name: &'static str,
    variant: &'static str,
    size: usize,
  ) -> Self::DiffTupleVariant;

  fn diff_newtype_struct<T>(
    self,
    name: &'static str,
    lhs: &T,
    rhs: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: Diff + ?Sized;

  fn diff_newtype_variant<T>(
    self,
    name: &'static str,
    variant: &'static str,
    lhs: &T,
    rhs: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: Diff + ?Sized;
}
