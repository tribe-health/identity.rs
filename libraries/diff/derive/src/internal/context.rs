use core::cell::RefCell;
use core::fmt::Display;
use quote::ToTokens;
use std::thread::panicking;

pub struct Context(RefCell<Option<Vec<syn::Error>>>);

impl Context {
  pub fn new() -> Self {
    Self(RefCell::new(Some(Vec::new())))
  }

  pub fn check(self) -> Result<(), Vec<syn::Error>> {
    let errors: Vec<syn::Error> = self.0.borrow_mut().take().unwrap();

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  pub fn error_spanned_by<A, T>(&self, object: A, message: T)
  where
    A: ToTokens,
    T: Display,
  {
    self
      .0
      .borrow_mut()
      .as_mut()
      .unwrap()
      .push(syn::Error::new_spanned(object.into_token_stream(), message));
  }

  pub fn syn_error(&self, error: syn::Error) {
    self.0.borrow_mut().as_mut().unwrap().push(error);
  }
}

impl Drop for Context {
  fn drop(&mut self) {
    if !panicking() && self.0.borrow().is_some() {
      panic!("forgot to check for errors");
    }
  }
}
