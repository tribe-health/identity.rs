use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use serde::ser::SerializeMap as _;
use serde::ser::SerializeSeq as _;
use serde::Serialize;
use serde::Serializer;
use serde_json::to_value;
use serde_json::Value;

use crate::error;
use crate::traits;

// =============================================================================
// =============================================================================

#[derive(Debug)]
pub struct Error(String);

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    f.write_str(&self.0)
  }
}

impl error::StdError for Error {}

impl error::Error for Error {
  #[inline]
  fn custom<T>(message: T) -> Self
  where
    T: Display,
  {
    Self(format!("{}", message))
  }
}

// =============================================================================
// =============================================================================

#[derive(Debug)]
pub struct JPatch;

impl JPatch {
  #[inline]
  pub fn diff<T>(lhs: &T, rhs: &T) -> Result<Command, Error>
  where
    T: traits::Diff + ?Sized,
  {
    if let Some(command) = traits::Diff::diff(lhs, rhs, Differ::new())? {
      Ok(command)
    } else {
      Ok(Command::Multiple(Vec::new()))
    }
  }
}

// =============================================================================
// =============================================================================

#[derive(Debug)]
pub enum Command {
  Insert(String, Value),
  Remove(String),
  Replace(String, Value),
  Multiple(Vec<Self>),
}

impl Serialize for Command {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::Insert(path, value) => {
        let mut out: S::SerializeMap = serializer.serialize_map(Some(3))?;
        out.serialize_entry("op", "add")?;
        out.serialize_entry("path", path)?;
        out.serialize_entry("value", value)?;
        out.end()
      }
      Self::Remove(path) => {
        let mut out: S::SerializeMap = serializer.serialize_map(Some(2))?;
        out.serialize_entry("op", "remove")?;
        out.serialize_entry("path", path)?;
        out.end()
      }
      Self::Replace(path, value) => {
        let mut out: S::SerializeMap = serializer.serialize_map(Some(3))?;
        out.serialize_entry("op", "replace")?;
        out.serialize_entry("path", path)?;
        out.serialize_entry("value", value)?;
        out.end()
      }
      Self::Multiple(list) => {
        let mut out: S::SerializeSeq = serializer.serialize_seq(Some(list.len()))?;
        for element in list {
          out.serialize_element(element)?;
        }
        out.end()
      }
    }
  }
}

#[derive(Debug)]
#[repr(transparent)]
struct CommandList(Vec<Command>);

impl CommandList {
  #[inline]
  fn with_capacity(capacity: usize) -> Self {
    Self(Vec::with_capacity(capacity))
  }

  #[inline]
  fn push(&mut self, command: Command) {
    match command {
      Command::Multiple(commands) => self.0.extend(commands),
      command => self.0.push(command),
    }
  }

  #[inline]
  fn into_command(self) -> Option<Command> {
    if self.0.is_empty() {
      None
    } else {
      Some(Command::Multiple(self.0))
    }
  }
}

// =============================================================================
// =============================================================================

#[derive(Debug)]
enum Path<'a> {
  Index(usize),
  Ident(&'a str),
}

impl Path<'_> {
  #[inline]
  fn size(&self) -> usize {
    match self {
      Self::Index(0) => 1,
      Self::Index(index) => ((*index as f64).log10() + 1.0).floor() as usize,
      Self::Ident(ident) => ident.len(),
    }
  }
}

// =============================================================================
// =============================================================================

struct Differ<'a> {
  size: usize,
  path: Option<Path<'a>>,
  root: Option<&'a Differ<'a>>,
}

impl Differ<'_> {
  #[inline]
  const fn new() -> Self {
    Self {
      size: 0,
      path: None,
      root: None,
    }
  }
}

impl<'a> Differ<'a> {
  #[inline]
  fn append<'b, 'c: 'b>(&'c self, path: Path<'b>) -> Differ<'b>
  where
    'a: 'b,
    'a: 'c,
  {
    Differ {
      size: self.size + path.size() + 1,
      path: Some(path),
      root: Some(self),
    }
  }

  #[inline]
  fn path(&self) -> String {
    let mut path: Vec<u8> = Vec::with_capacity(self.size);

    self.append_to(&mut path);

    // SAFETY: Path segments are utf8 srings OR formatted integers
    unsafe { String::from_utf8_unchecked(path) }
  }

  #[inline]
  fn append_to(&self, output: &mut Vec<u8>) {
    if let Some(root) = self.root {
      root.append_to(output);
    }

    if let Some(ref path) = self.path {
      output.push(b'/');

      match path {
        Path::Index(index) => {
          let _ = itoa::write(output, *index);
        }
        Path::Ident(ident) => {
          output.extend_from_slice(ident.as_bytes());
        }
      }
    }
  }
}

impl Debug for Differ<'_> {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    Debug::fmt(&self.path(), f)
  }
}

impl<'a> traits::Differ for Differ<'a> {
  type Ok = Option<Command>;
  type Error = Error;

  type DiffMap = DiffMap<'a>;
  type DiffSeq = DiffSeq<'a>;
  type DiffStruct = DiffStruct<'a>;
  type DiffStructVariant = DiffStruct<'a>;
  type DiffTuple = DiffSeq<'a>;
  type DiffTupleStruct = DiffSeq<'a>;
  type DiffTupleVariant = DiffSeq<'a>;

  #[inline]
  fn difference<T>(self, _lhs: &T, rhs: &T) -> Result<Self::Ok, Self::Error>
  where
    T: Serialize + ?Sized,
  {
    Ok(Some(Command::Replace(
      self.path(),
      to_value(rhs).map_err(error::Error::custom)?,
    )))
  }

  #[inline]
  fn same<T>(self, _lhs: &T, _rhs: &T) -> Result<Self::Ok, Self::Error>
  where
    T: Serialize + ?Sized,
  {
    Ok(None)
  }

  #[inline]
  fn diff_map(self) -> Self::DiffMap {
    DiffMap {
      root: self,
      list: CommandList::with_capacity(0),
    }
  }

  #[inline]
  fn diff_seq(self, size: Option<usize>) -> Self::DiffSeq {
    DiffSeq {
      root: self,
      list: CommandList::with_capacity(size.unwrap_or(0)),
      index: 0,
    }
  }

  #[inline]
  fn diff_struct(self, _name: &'static str, size: usize) -> Self::DiffStruct {
    DiffStruct {
      root: self,
      list: CommandList::with_capacity(size),
    }
  }

  #[inline]
  fn diff_struct_variant(
    self,
    _name: &'static str,
    _variant: &'static str,
    size: usize,
  ) -> Self::DiffStructVariant {
    DiffStruct {
      root: self,
      list: CommandList::with_capacity(size),
    }
  }

  #[inline]
  fn diff_tuple(self, size: usize) -> Self::DiffTuple {
    DiffSeq {
      root: self,
      list: CommandList::with_capacity(size),
      index: 0,
    }
  }

  #[inline]
  fn diff_tuple_struct(self, _name: &'static str, size: usize) -> Self::DiffTupleStruct {
    DiffSeq {
      root: self,
      list: CommandList::with_capacity(size),
      index: 0,
    }
  }

  #[inline]
  fn diff_tuple_variant(
    self,
    _name: &'static str,
    _variant: &'static str,
    size: usize,
  ) -> Self::DiffTupleVariant {
    DiffSeq {
      root: self,
      list: CommandList::with_capacity(size),
      index: 0,
    }
  }

  #[inline]
  fn diff_newtype_struct<T>(
    self,
    _name: &'static str,
    lhs: &T,
    rhs: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    traits::Diff::diff(lhs, rhs, self)
  }

  #[inline]
  fn diff_newtype_variant<T>(
    self,
    _name: &'static str,
    _variant: &'static str,
    lhs: &T,
    rhs: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    traits::Diff::diff(lhs, rhs, self)
  }
}

// =============================================================================
//
// =============================================================================

#[derive(Debug)]
struct DiffMap<'a> {
  root: Differ<'a>,
  list: CommandList,
}

impl<'a> traits::DiffMap for DiffMap<'a> {
  type Ok = Option<Command>;
  type Error = Error;

  #[inline]
  fn visit<T, U>(&mut self, _key: &T, _lhs: &U, _rhs: &U) -> Result<(), Self::Error>
  where
    T: Display + ?Sized,
    U: traits::Diff + ?Sized,
  {
    todo!("DiffMap::visit")
  }

  #[inline]
  fn visit_lhs<T, U>(&mut self, _key: &T, _lhs: &U) -> Result<(), Self::Error>
  where
    T: Display + ?Sized,
    U: traits::Diff + ?Sized,
  {
    todo!("DiffMap::visit_lhs")
  }

  #[inline]
  fn visit_rhs<T, U>(&mut self, _key: &T, _rhs: &U) -> Result<(), Self::Error>
  where
    T: Display + ?Sized,
    U: traits::Diff + ?Sized,
  {
    todo!("DiffMap::visit_rhs")
  }

  #[inline]
  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(self.list.into_command())
  }
}

// =============================================================================
//
// =============================================================================

#[derive(Debug)]
struct DiffSeq<'a> {
  root: Differ<'a>,
  list: CommandList,
  index: usize,
}

impl<'a> traits::DiffSeq for DiffSeq<'a> {
  type Ok = Option<Command>;
  type Error = Error;

  #[inline]
  fn visit<T>(&mut self, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    let dif: Differ = self.root.append(Path::Index(self.index));

    if let Some(out) = traits::Diff::diff(lhs, rhs, dif)? {
      self.list.push(out);
    }

    self.index += 1;

    Ok(())
  }

  #[inline]
  fn visit_lhs<T>(&mut self, _lhs: &T) -> Result<(), Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    let dif: Differ = self.root.append(Path::Index(self.index));

    self.list.push(Command::Remove(dif.path()));
    self.index += 1;

    Ok(())
  }

  #[inline]
  fn visit_rhs<T>(&mut self, rhs: &T) -> Result<(), Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    let dif: Differ = self.root.append(Path::Index(self.index));

    self.list.push(Command::Insert(
      dif.path(),
      to_value(rhs).map_err(error::Error::custom)?,
    ));

    self.index += 1;

    Ok(())
  }

  #[inline]
  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(self.list.into_command())
  }
}

// =============================================================================
//
// =============================================================================

#[derive(Debug)]
struct DiffStruct<'a> {
  root: Differ<'a>,
  list: CommandList,
}

impl<'a> traits::DiffStruct for DiffStruct<'a> {
  type Ok = Option<Command>;
  type Error = Error;

  #[inline]
  fn visit<T>(&mut self, key: &'static str, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    let dif: Differ = self.root.append(Path::Ident(key));

    if let Some(out) = traits::Diff::diff(lhs, rhs, dif)? {
      self.list.push(out);
    }

    Ok(())
  }

  #[inline]
  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(self.list.into_command())
  }
}

// =============================================================================
//
// =============================================================================

impl<'a> traits::DiffStructVariant for DiffStruct<'a> {
  type Ok = Option<Command>;
  type Error = Error;

  #[inline]
  fn visit<T>(&mut self, key: &'static str, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    traits::DiffStruct::visit(self, key, lhs, rhs)
  }

  #[inline]
  fn end(self) -> Result<Self::Ok, Self::Error> {
    traits::DiffStruct::end(self)
  }
}

// =============================================================================
//
// =============================================================================

impl<'a> traits::DiffTuple for DiffSeq<'a> {
  type Ok = Option<Command>;
  type Error = Error;

  #[inline]
  fn visit<T>(&mut self, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    traits::DiffSeq::visit(self, lhs, rhs)
  }

  #[inline]
  fn end(self) -> Result<Self::Ok, Self::Error> {
    traits::DiffSeq::end(self)
  }
}

// =============================================================================
//
// =============================================================================

impl<'a> traits::DiffTupleStruct for DiffSeq<'a> {
  type Ok = Option<Command>;
  type Error = Error;

  #[inline]
  fn visit<T>(&mut self, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    traits::DiffSeq::visit(self, lhs, rhs)
  }

  #[inline]
  fn end(self) -> Result<Self::Ok, Self::Error> {
    traits::DiffSeq::end(self)
  }
}

// =============================================================================
//
// =============================================================================

impl<'a> traits::DiffTupleVariant for DiffSeq<'a> {
  type Ok = Option<Command>;
  type Error = Error;

  #[inline]
  fn visit<T>(&mut self, lhs: &T, rhs: &T) -> Result<(), Self::Error>
  where
    T: traits::Diff + ?Sized,
  {
    traits::DiffSeq::visit(self, lhs, rhs)
  }

  #[inline]
  fn end(self) -> Result<Self::Ok, Self::Error> {
    traits::DiffSeq::end(self)
  }
}
