#[cfg(feature = "smallvec")]
pub mod smallvec;
pub mod vec;

use core::{fmt, slice};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{self, Bound, Range, RangeBounds, RangeTo};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifyError<const M: usize>;

impl<const M: usize> Display for ModifyError<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "operation would reduce length below minimum required {M}",
        )
    }
}

impl<const M: usize> Error for ModifyError<M> {}

#[inline]
#[track_caller]
fn slice_range<R>(range: &R, bounds: RangeTo<usize>) -> Range<usize>
where
    R: RangeBounds<usize>,
{
    let len = bounds.end;

    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(start) => start
            .checked_add(1)
            .ok_or("attempted to index slice from after maximum usize")
            .unwrap(),
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        Bound::Included(end) => end
            .checked_add(1)
            .ok_or("attempted to index slice up to maximum usize")
            .unwrap(),
        Bound::Excluded(&end) => end,
        Bound::Unbounded => len,
    };

    assert!(
        start <= end,
        "slice index starts at {start} but ends at {end}"
    );
    assert!(
        end <= len,
        "range end index {end} out of range for slice of length {len}"
    );

    Range { start, end }
}
