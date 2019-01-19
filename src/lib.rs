//! This crate exposes a single data type, [`SortedVec`]. It provides a lookup table that
//! has quicker lookups than regular `Vec`s, `O(log(n))` vs `O(n)`,
//! and is simpler and more memory efficient than hashmaps. It is ideal for (very) small
//! lookup tables where additions and deletions are infrequent.
//!
//! # Example
//! ```
//! use sortedvec::SortedVec;
//!
//! let unsorted = vec![3, 5, 0, 10, 7, 1];
//! let sorted = SortedVec::from_vec(unsorted.clone(), |x| x);
//!
//! // linear search (slow!)
//! let unsorted_contains_six: Option<_> = unsorted.iter().find(|&x| *x == 6);
//! assert!(unsorted_contains_six.is_none());
//!
//! // binary search (fast!)
//! let sorted_contains_six: Option<_> = sorted.find(&6);
//! assert!(sorted_contains_six.is_none());
//! ```

use std::borrow::Borrow;
use std::hash::{self, Hash};
use std::iter::Extend;
use std::ops::Deref;

#[macro_export]
macro_rules! sortedvec {
    ( $ ( $ x : expr ) , * ) => {
        {
            let inner = vec![ $ ( $x, )* ];
            $crate::from_vec(inner, |x| x)
        }
    }
}

pub fn from_vec<T, F, K>(v: Vec<T>, f: F) -> SortedVec<T, F>
where
    F: Clone + for<'t> Fn(&'t T) -> &'t K,
    K: Ord + Eq + ?Sized,
{
    SortedVec::from_vec(v, f)
}

/// A `Vec` wrapper type for orderable elements, providing `log(N)` lookups.
///
/// `SortedVec` is a `Vec` whose elements are sorted with respect some comperator
/// function. This provides some behaviour in between regular vectors and hashmaps, namely:
/// * a compact memory representation
/// * fast iteration
/// * semi-fast lookups: `O(log(N))`, compared to a hashmap's `O(1)` and a vector's `O(n)`
/// * slow insertions and deletions: `O(n)`
///
/// Its main use case is for small lookup tables where inserts and deletions are infrequent.
///
/// # Example
/// ```
/// use sortedvec::SortedVec;
///
/// struct A {
///     val: f64,
///     key: u32,
/// }
///
/// let mut sv = SortedVec::new(|a: &A| &a.key);
/// sv.insert(A { val: 3.14, key: 0 });
/// sv.insert(A { val: 0.00, key: 10 });
/// sv.insert(A { val: 5.00, key: 4 });
///
/// assert_eq!(3, sv.len());
/// assert!(sv.find(&5).is_none());
///
/// let search_result = sv.find(&4).unwrap();
/// assert_eq!(5.00, search_result.val);
/// ```
pub struct SortedVec<T, F> {
    inner: Vec<T>,
    comp: F,
}

impl<T, F, K> SortedVec<T, F>
where
    F: Clone + for<'t> Fn(&'t T) -> &'t K,
    K: Ord + Eq + ?Sized,
{
    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated `Self`. `self` contains elements `[0, at)`,
    /// and the returned `Self` contains elements `[at, len)`.
    ///
    /// Note that the capacity of `self` does not change.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    pub fn split_off(&mut self, at: usize) -> Self {
        let other_inner = self.inner.split_off(at);
        Self {
            inner: other_inner,
            comp: self.comp.clone(),
        }
    }
}

impl<T, F, K> SortedVec<T, F>
where
    F: for<'t> Fn(&'t T) -> &'t K,
    K: Ord + Eq + ?Sized,
{
    /// Creates a new, empty `SortedVec`. Does not allocate until elements
    /// are inserted.
    pub fn new(comp: F) -> Self {
        Self {
            inner: Vec::new(),
            comp,
        }
    }

    /// Creates a sorted vector from an existing vector, which does
    /// not need to be sorted beforehand, and a comparison function.
    ///
    /// # Example
    ///
    /// ```
    /// use sortedvec::*;
    ///
    /// let v = vec!["hello".to_string(), "world".to_string()];
    /// let sv = SortedVec::from_vec(v, |x| &x[1..]);
    /// ```
    pub fn from_vec(mut vec: Vec<T>, comp: F) -> Self {
        vec.sort_unstable_by(|ref a, ref b| {
            let lhs = comp(a);
            let rhs = comp(b);
            lhs.cmp(&rhs)
        });

        let sorted = Self { inner: vec, comp };

        debug_assert!(sorted.is_sorted());

        sorted
    }

    /// Inserts a new value into the vector, maintaining the internal
    /// order invariant. Note that this is an `O(n)` operation.
    pub fn insert(&mut self, val: T) {
        let ref key = (self.comp)(&val);
        let search = self
            .inner
            .binary_search_by(|probe| (self.comp)(probe).cmp(key));
        let idx = match search {
            Ok(i) | Err(i) => i,
        };
        self.inner.insert(idx, val);

        debug_assert!(self.is_sorted());
    }

    /// Provides access to the internal comperator function.
    pub fn comperator(&self) -> &F {
        &self.comp
    }

    /// Tries to find an element in the vector with the given 'key'. It has
    /// logarithmic worst case time complexity. The
    /// elements' keys are computed using the internal comperator function,
    /// which is exposed through the [`SortedVec::comperator`] method.
    ///
    /// #Example
    ///
    /// ```
    /// use sortedvec::*;
    ///
    /// let sv = sortedvec![1, 5, 3, 10];
    /// let res = sv.find(&5).unwrap();
    /// assert_eq!(&5, res);
    /// ```
    pub fn find(&self, key: &K) -> Option<&T> {
        self.inner
            .binary_search_by(|probe| (self.comp)(probe).cmp(key))
            .ok()
            .and_then(|idx| self.inner.get(idx))
    }

    /// Checks whether there is a value with that key in the vector. This is
    /// done in `O(log(n))` time.
    pub fn contains(&self, key: &K) -> bool {
        self.inner
            .binary_search_by(|probe| (self.comp)(probe).cmp(key))
            .is_ok()
    }

    /// Destructs the `SortedVec` to its inner `Vec`, which is guaranteed
    /// to be sorted with respect to the comperator function.
    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    /// #[unstable(reason = "indices are opaque for this data structure")]
    pub fn remove(&mut self, index: usize) -> T {
        self.inner.remove(index)
    }

    /// Shortens the vector, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater than the vector's current length, this has no
    /// effect.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the vector.
    pub fn truncate(&mut self, len: usize) {
        self.inner.truncate(len)
    }

    /// Removes all elements but one that resolve to the same key generated
    /// by the internal key function.
    pub fn dedup_by_key(&mut self) {
        let mut dummy = Vec::new();
        std::mem::swap(&mut dummy, &mut self.inner);

        dummy.dedup_by(|a, b| (self.comp)(a) == (self.comp)(b));

        std::mem::swap(&mut dummy, &mut self.inner);
    }

    /// Removes the last element from a vector and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    // internal methods
    fn sort(&mut self) {
        // need to swap out the internal because we aren't allowed
        // to mutate it while we borrow the comperator function.
        // slightly suboptimal, but it's (probably) the best we can
        // do without unsafe.
        let mut dummy = Vec::new();
        std::mem::swap(&mut dummy, &mut self.inner);

        dummy.sort_unstable_by(|ref a, ref b| {
            let lhs = (&self.comp)(a);
            let rhs = (&self.comp)(b);
            lhs.cmp(&rhs)
        });

        std::mem::swap(&mut dummy, &mut self.inner);

        debug_assert!(self.is_sorted());
    }

    // this should *always* return true
    fn is_sorted(&self) -> bool {
        if self.inner.len() == 0 {
            return true;
        }

        for i in 0..(self.inner.len() - 1) {
            if (self.comp)(&self.inner[i]) > (self.comp)(&self.inner[i + 1]) {
                return false;
            }
        }

        true
    }
}

/// An iterator that moves out of a sorted vector.
pub struct IntoIter<T> {
    inner: std::vec::IntoIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T, F> IntoIterator for SortedVec<T, F> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner: self.inner.into_iter(),
        }
    }
}

impl<T, F, K> Extend<T> for SortedVec<T, F>
where
    F: for<'t> Fn(&'t T) -> &'t K,
    K: Ord + Eq + ?Sized,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.inner.extend(iter);
        self.sort();
    }
}

// Do we want to distinguish between two sorted vecs with the
// exact same underlying vec, but a different comp? Probably not,
// but we may be breaking an invariant here.
impl<T, F> Hash for SortedVec<T, F>
where
    T: Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&*self.inner, state)
    }
}

impl<T, F> Deref for SortedVec<T, F> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, F> Borrow<[T]> for SortedVec<T, F> {
    fn borrow(&self) -> &[T] {
        &self.inner
    }
}

impl<T, F> AsRef<[T]> for SortedVec<T, F> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}

impl<T, F> AsRef<Vec<T>> for SortedVec<T, F> {
    fn as_ref(&self) -> &Vec<T> {
        &self.inner
    }
}

#[cfg(test)]
mod tests {}
