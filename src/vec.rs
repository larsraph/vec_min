use std::borrow::{Borrow, BorrowMut, Cow};
use std::collections::TryReserveError;
use std::iter::repeat_with;
use std::mem::MaybeUninit;
use std::ops::{Bound, Deref, RangeBounds};
use std::{slice, vec};

use crate::MinLenError;

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecMin<T, const M: usize> {
    vec: Vec<T>,
}

// --- Custom ---
impl<T, const M: usize> VecMin<T, M> {
    /// Returns the minimum length of the vector.
    #[inline]
    pub const fn min_len(&self) -> usize {
        M
    }

    /// Returns a slice to the first `M` elements of the vector, which are guaranteed to exist.
    #[inline]
    pub const fn min_slice(&self) -> &[T; M] {
        unsafe { &*(self.vec.as_ptr() as *const [T; M]) }
    }
}

/// --- Iterators ---
impl<T, const M: usize> IntoIterator for VecMin<T, M> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a, T: 'a, const M: usize> IntoIterator for &'a VecMin<T, M> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl<'a, T: 'a, const M: usize> IntoIterator for &'a mut VecMin<T, M> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter_mut()
    }
}

// --- Constructors & Destructors & Conversion ---
impl<T, const M: usize> VecMin<T, M> {
    /// Creates a new `VecMin` from a `Vec`
    ///
    /// # Safety
    /// - The length of the provided `Vec` must be at least `M`.
    #[inline]
    pub const unsafe fn new_unchecked(vec: Vec<T>) -> Self {
        Self { vec }
    }

    /// Creates a new `VecMin` from a `Vec`, returning an error if the length of the provided `Vec` is less than `M`.
    #[inline]
    pub const fn new(vec: Vec<T>) -> Result<Self, Vec<T>> {
        if vec.len() >= M {
            Ok(unsafe { Self::new_unchecked(vec) })
        } else {
            Err(vec)
        }
    }

    #[inline]
    fn _collect(iter: impl IntoIterator<Item = T>, cap: usize) -> Result<Self, Vec<T>> {
        let iter = iter.into_iter();
        let (hint, _) = iter.size_hint();

        let mut vec = Vec::with_capacity(cap.max(hint));
        vec.extend(iter);

        Self::new(vec)
    }

    /// Creates a new `VecMin` from an iterator, returning an error if the length of the collected `Vec` is less than `M`.
    #[inline]
    pub fn collect(iter: impl IntoIterator<Item = T>) -> Result<Self, Vec<T>> {
        Self::_collect(iter, M)
    }

    /// Creates a new `VecMin` from an iterator, returning an error if the length of the collected `Vec` is less than `M`.
    /// The provided `extra` capacity is added to the minimum capacity of `M` when collecting the iterator.
    #[inline]
    pub fn collect_with_capacity(
        iter: impl IntoIterator<Item = T>,
        extra: usize,
    ) -> Result<Self, Vec<T>> {
        Self::_collect(iter, M + extra)
    }

    /// Returns the inner `Vec`, consuming the `VecMin`.
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.vec
    }

    /// See [`Vec::into_boxed_slice`].
    #[inline]
    pub fn into_boxed_slice(self) -> Box<[T]> {
        self.vec.into_boxed_slice()
    }

    /// See [`Vec::leak`].
    #[inline]
    pub fn leak(self) -> &'static mut [T] {
        self.vec.leak()
    }
}

impl<T: Default, const M: usize> Default for VecMin<T, M> {
    #[inline]
    fn default() -> Self {
        Self {
            vec: repeat_with(T::default).take(M).collect(),
        }
    }
}

impl<T, const M: usize> TryFrom<Vec<T>> for VecMin<T, M> {
    type Error = Vec<T>;

    #[inline]
    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(vec)
    }
}

impl<T, const M: usize> From<VecMin<T, M>> for Vec<T> {
    #[inline]
    fn from(vec_min: VecMin<T, M>) -> Self {
        vec_min.vec
    }
}

impl<T, const M: usize> TryFrom<Box<[T]>> for VecMin<T, M> {
    type Error = Vec<T>;

    #[inline]
    fn try_from(boxed_slice: Box<[T]>) -> Result<Self, Self::Error> {
        let vec = boxed_slice.into_vec();
        Self::new(vec)
    }
}

impl<T, const M: usize> From<VecMin<T, M>> for Box<[T]> {
    #[inline]
    fn from(vec_min: VecMin<T, M>) -> Self {
        vec_min.into_boxed_slice()
    }
}

impl<T: Clone, const M: usize> TryFrom<&[T]> for VecMin<T, M> {
    type Error = Vec<T>;

    #[inline]
    fn try_from(slice: &[T]) -> Result<Self, Self::Error> {
        Self::new(slice.to_vec())
    }
}

impl<T: Clone, const M: usize> TryFrom<&mut [T]> for VecMin<T, M> {
    type Error = Vec<T>;

    #[inline]
    fn try_from(slice: &mut [T]) -> Result<Self, Self::Error> {
        Self::new(slice.to_vec())
    }
}

impl<T: Clone, const M: usize> TryFrom<Cow<'_, [T]>> for VecMin<T, M> {
    type Error = Vec<T>;

    #[inline]
    fn try_from(cow: Cow<'_, [T]>) -> Result<Self, Self::Error> {
        Self::new(cow.into_owned())
    }
}

impl<T, const N: usize, const M: usize> TryFrom<[T; N]> for VecMin<T, M> {
    type Error = Vec<T>;

    #[inline]
    fn try_from(array: [T; N]) -> Result<Self, Self::Error> {
        Self::new(array.into())
    }
}

// --- Immutable Len Access ---
impl<T, const M: usize> Deref for VecMin<T, M> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

// --- Mutable Len Access ---

// -- Not Len Decreasing --

// - Cap -
impl<T, const M: usize> VecMin<T, M> {
    /// See [`Vec::reserve`].
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
    }

    /// See [`Vec::reserve_exact`].
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.vec.reserve_exact(additional);
    }

    /// See [`Vec::try_reserve`].
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.vec.try_reserve(additional)
    }

    /// See [`Vec::try_reserve_exact`].
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.vec.try_reserve_exact(additional)
    }

    /// See [`Vec::shrink_to_fit`].
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.vec.shrink_to_fit();
    }

    /// See [`Vec::shrink_to`].
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.vec.shrink_to(min_capacity);
    }
}

// - View -
impl<T, const M: usize> VecMin<T, M> {
    /// See [`Vec::as_mut_slice`].
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        self.vec.as_mut_slice()
    }

    /// See [`Vec::as_mut_ptr`].
    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.vec.as_mut_ptr()
    }

    /// See [`Vec::spare_capacity_mut`].
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        self.vec.spare_capacity_mut()
    }
}

impl<T, const M: usize> AsRef<[T]> for VecMin<T, M> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.vec
    }
}

impl<T, const M: usize> AsMut<[T]> for VecMin<T, M> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.vec
    }
}

impl<T, const M: usize> Borrow<[T]> for VecMin<T, M> {
    #[inline]
    fn borrow(&self) -> &[T] {
        &self.vec
    }
}

impl<T, const M: usize> BorrowMut<[T]> for VecMin<T, M> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self.vec
    }
}

// - Increasing len -
impl<T, const M: usize> VecMin<T, M> {
    /// See [`Vec::push`].
    #[inline]
    pub fn push(&mut self, item: T) {
        self.vec.push(item);
    }

    /// See [`Vec::insert`].
    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        self.vec.insert(index, element);
    }

    /// See [`Vec::append`].
    #[inline]
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.vec.append(other);
    }

    /// See [`Vec::extend_from_slice`].
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        self.vec.extend_from_slice(other);
    }

    /// See [`Vec::extend_from_within`].
    #[inline]
    pub fn extend_from_within<R>(&mut self, range: R)
    where
        R: std::ops::RangeBounds<usize>,
        T: Clone,
    {
        self.vec.extend_from_within(range);
    }
}

impl<T, const M: usize> Extend<T> for VecMin<T, M> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.vec.extend(iter);
    }
}

impl<'a, T: Copy, const M: usize> Extend<&'a T> for VecMin<T, M> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.vec.extend(iter);
    }
}

// -- Len Decreasing --
impl<T, const M: usize> VecMin<T, M> {
    /// See [`Vec::pop`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn pop(&mut self) -> Result<Option<T>, MinLenError> {
        if self.vec.len() > M {
            Ok(self.vec.pop())
        } else {
            Err(MinLenError::BelowMinimum)
        }
    }

    /// See [`Vec::pop`]. Pops an element from the vector if the length of the vector is greater than `M`, otherwise does nothing and returns `None`.
    #[inline]
    pub fn pop_to_min(&mut self) -> Option<T> {
        if self.vec.len() > M {
            self.vec.pop()
        } else {
            None
        }
    }

    /// See [`Vec::remove`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn remove(&mut self, index: usize) -> Result<T, MinLenError> {
        if self.vec.len() > M {
            Ok(self.vec.remove(index))
        } else {
            Err(MinLenError::BelowMinimum)
        }
    }

    /// See [`Vec::swap_remove`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn swap_remove(&mut self, index: usize) -> Result<T, MinLenError> {
        if self.vec.len() > M {
            Ok(self.vec.swap_remove(index))
        } else {
            Err(MinLenError::BelowMinimum)
        }
    }

    /// See [`Vec::truncate`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn truncate(&mut self, len: usize) -> Result<(), MinLenError> {
        if len >= M {
            Ok(self.vec.truncate(len))
        } else {
            Err(MinLenError::BelowMinimum)
        }
    }

    /// See [`Vec::truncate`]. Truncates the vector to `len` if `len` is greater than or equal to `M`, otherwise truncates the vector to `M`.
    #[inline]
    pub fn truncate_or_min(&mut self, len: usize) {
        self.vec.truncate(len.max(M))
    }

    /// See [`Vec::truncate`]. Truncates the vector to `M`.
    #[inline]
    pub fn truncate_to_min(&mut self) {
        self.vec.truncate(M);
    }

    /// See [`Vec::resize`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn resize(&mut self, new_len: usize, value: T) -> Result<(), MinLenError>
    where
        T: Clone,
    {
        if new_len >= M {
            Ok(self.vec.resize(new_len, value))
        } else {
            Err(MinLenError::BelowMinimum)
        }
    }

    /// See [`Vec::resize`]. Resizes the vector to `new_len` if `new_len` is greater than or equal to `M`, otherwise resizes the vector to `M`.
    #[inline]
    #[must_use]
    pub fn resize_or_min(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.vec.resize(new_len.max(M), value);
    }

    /// See [`Vec::resize_with`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[inline]
    #[must_use]
    pub fn resize_with<F>(&mut self, new_len: usize, generator: F) -> Result<(), MinLenError>
    where
        F: FnMut() -> T,
    {
        if new_len >= M {
            Ok(self.vec.resize_with(new_len, generator))
        } else {
            Err(MinLenError::BelowMinimum)
        }
    }

    /// See [`Vec::resize_with`]. Resizes the vector to `new_len` if `new_len` is greater than or equal to `M`, otherwise resizes the vector to `M`.
    #[inline]
    #[must_use]
    pub fn resize_or_min_with<F>(&mut self, new_len: usize, generator: F)
    where
        F: FnMut() -> T,
    {
        self.vec.resize_with(new_len.max(M), generator);
    }

    /// See [`Vec::drain`]. Returns an error if the operation would reduce the length of the vector below `M`.
    #[must_use]
    pub fn drain<R>(&mut self, range: R) -> Result<vec::Drain<'_, T>, MinLenError>
    where
        R: RangeBounds<usize>,
    {
        let drain_len = range_len(&range, self.vec.len());
        let final_len = self.vec.len() - drain_len;

        if final_len >= M {
            Ok(self.vec.drain(range))
        } else {
            Err(MinLenError::BelowMinimum)
        }
    }

    /// See [`Vec::splice`]. Drains the specified range if draining it would not reduce the length of the vector below `M`, otherwise does nothing and returns an empty iterator.
    #[must_use]
    pub fn splice<R, I>(
        &mut self,
        range: R,
        replace_with: I,
    ) -> Result<vec::Splice<'_, I::IntoIter>, MinLenError>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T, IntoIter: ExactSizeIterator>,
    {
        let replace_with = replace_with.into_iter();

        let gain = replace_with.len();
        let loss = range_len(&range, self.vec.len());
        let final_len = self.vec.len() + gain - loss;

        if final_len >= M {
            Ok(self.vec.splice(range, replace_with))
        } else {
            Err(MinLenError::BelowMinimum)
        }
    }
}

#[inline]
fn range_len(range: &impl RangeBounds<usize>, max: usize) -> usize {
    let start = match range.start_bound() {
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n.saturating_add(1),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&n) => n.saturating_add(1),
        Bound::Excluded(&n) => n,
        Bound::Unbounded => max,
    };

    end.saturating_sub(start)
}
