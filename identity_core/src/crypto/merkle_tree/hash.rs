use core::cmp::Ordering;
use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use core::marker::PhantomData;
use digest::Output;
use digest::Digest;
use digest::generic_array::typenum::Unsigned;
use serde::Serialize;
use serde::Deserialize;
use serde::Serializer;
use serde::de::Unexpected;
use serde::de::Deserializer;
use serde::de::Visitor;
use serde::de::Error;

use crate::utils::encode_hex;
use crate::utils::decode_hex;

pub struct Hash<D>(Output<D>)
where
  D: Digest;

impl<D: Digest> Hash<D> {
  pub fn from_slice(slice: &[u8]) -> Option<Self> {
    if slice.len() != D::OutputSize::USIZE {
      return None;
    }

    let mut this: Self = Self::default();

    this.0.copy_from_slice(slice);

    Some(this)
  }
}

impl<D: Digest> Clone for Hash<D>
where
  Output<D>: Clone,
{
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<D: Digest> Copy for Hash<D> where Output<D>: Copy {}

impl<D: Digest> PartialEq for Hash<D>
where
  Output<D>: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    self.0.eq(&other.0)
  }
}

impl<D: Digest> Eq for Hash<D> where Output<D>: Eq {}

impl<D: Digest> PartialOrd for Hash<D>
where
  Output<D>: PartialOrd,
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.0.partial_cmp(&other.0)
  }
}

impl<D: Digest> Ord for Hash<D>
where
  Output<D>: Ord,
{
  fn cmp(&self, other: &Self) -> Ordering {
    self.0.cmp(&other.0)
  }
}

impl<D: Digest> Debug for Hash<D> {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    f.write_str(&encode_hex(self))
  }
}

impl<D: Digest> Default for Hash<D> {
  fn default() -> Self {
    Self(Output::<D>::default())
  }
}

impl<D: Digest> AsRef<[u8]> for Hash<D> {
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref()
  }
}

impl<D: Digest> From<Output<D>> for Hash<D> {
  fn from(other: Output<D>) -> Self {
    Self(other)
  }
}

impl<D: Digest> Serialize for Hash<D> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer
  {
    serializer.serialize_str(&encode_hex(self))
  }
}

impl<'de, D: Digest> Deserialize<'de> for Hash<D> {
  fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
  where
    DE: Deserializer<'de>,
  {
    struct __Visitor<D> {
      marker: PhantomData<D>,
    }

    impl<'de, D: Digest> Visitor<'de> for __Visitor<D> {
      type Value = Hash<D>;

      fn expecting(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("a base16-encoded string")
      }

      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: Error {
        Hash::from_slice(&decode_hex(value).map_err(Error::custom)?)
          .ok_or_else(|| Error::invalid_value(Unexpected::Str(value), &self))
      }
    }

    deserializer.deserialize_str(__Visitor {
      marker: PhantomData
    })
  }
}
