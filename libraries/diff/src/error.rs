use crate::lib::*;

#[cfg(not(feature = "std"))]
mod std_error {
  use crate::lib::*;

  pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
      None
    }
  }
}

#[cfg(feature = "std")]
#[doc(no_inline)]
pub use std::error::Error as StdError;
#[cfg(not(feature = "std"))]
#[doc(no_inline)]
pub use std_error::Error as StdError;

macro_rules! impl_error_trait {
  (Error: Sized $(+ $($supertrait:ident)::+)*) => {
    pub trait Error: Sized $(+ $($supertrait)::+)* {
      fn custom<T>(message: T) -> Self where T: Display;
    }
  };
}

#[cfg(feature = "std")]
impl_error_trait!(Error: Sized + StdError);

#[cfg(not(feature = "std"))]
impl_error_trait!(Error: Sized + Debug + Display);
