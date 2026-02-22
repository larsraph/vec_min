#[cfg(feature = "smallvec")]
pub mod smallvec;
pub mod vec;

use core::fmt;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MinLenError {
    BelowMinimum,
}

impl Display for MinLenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MinLenError::BelowMinimum => {
                write!(
                    f,
                    "operation would reduce SmallVecMin below its minimum length"
                )
            }
        }
    }
}

impl Error for MinLenError {}
