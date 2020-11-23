use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use syn::Ident;
use syn::Path;

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

impl PartialEq for Symbol {
  fn eq(&self, word: &Self) -> bool {
    self.0 == word.0
  }
}

impl PartialEq<Symbol> for Ident {
  fn eq(&self, word: &Symbol) -> bool {
    self == word.0
  }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
  fn eq(&self, word: &Symbol) -> bool {
    *self == word.0
  }
}

impl PartialEq<Symbol> for Path {
  fn eq(&self, word: &Symbol) -> bool {
    self.is_ident(word.0)
  }
}

impl<'a> PartialEq<Symbol> for &'a Path {
  fn eq(&self, word: &Symbol) -> bool {
    self.is_ident(word.0)
  }
}

impl Display for Symbol {
  fn fmt(&self, f: &mut Formatter) -> Result {
    f.write_str(self.0)
  }
}

pub const GETTER: Symbol = Symbol("getter");
pub const PATCH: Symbol = Symbol("patch");
pub const FROM: Symbol = Symbol("from");
pub const INTO: Symbol = Symbol("into");
pub const RENAME: Symbol = Symbol("rename");
pub const RENAME_ALL: Symbol = Symbol("rename_all");
pub const SKIP: Symbol = Symbol("skip");
pub const SKIP_DIFF: Symbol = Symbol("skip_diff");
pub const SKIP_MERGE: Symbol = Symbol("skip_merge");
pub const TRANSPARENT: Symbol = Symbol("transparent");
pub const TRY_FROM: Symbol = Symbol("try_from");
pub const UNTAGGED: Symbol = Symbol("untagged");
