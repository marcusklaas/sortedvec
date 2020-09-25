//! This crate exposes macros that generate data structures
//! on `Ord` keys that provide faster lookups than regular `Vec`s (`O(log(n))` vs `O(n)`)
//! and is simpler and more memory efficient than hashmaps. It is ideal for small
//! lookup tables where insertions and deletions are infrequent.

//!
//! # Example
//! ```
//! use sortedvec::sortedvec;
//!
//! sortedvec! {
//!     struct SortedVec {
//!         fn derive_key(x: &u32) -> u32 { *x }
//!     }
//! }
//!
//! let unsorted = vec![3, 5, 0, 10, 7, 1];
//! let sorted = SortedVec::from(unsorted.clone());
//!
//! // linear search (slow!)
//! let unsorted_contains_six: Option<_> = unsorted.iter().find(|&x| *x == 6);
//! assert!(unsorted_contains_six.is_none());
//!
//! // binary search (fast!)
//! let sorted_contains_six: Option<_> = sorted.find(&6);
//! assert!(sorted_contains_six.is_none());
//! ```

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

/// An example of a data structure defined using the `sortedvec!` macro.
pub mod example;

/// A macro that defines a sorted vector data structure.
///
/// The generated struct is specific to the given keys and value types. To create the struct,
/// four bits are required:
/// - a struct name,
/// - a value type,
/// - a key type. Since we will sort on these internally, this type must implement `Ord`,
/// - a key extraction function of type `FnMut(&T) -> K`.
///
/// It matches the following input:
/// ```text
/// $(#[$attr:meta])*
/// $v:vis struct $name:ident {
///     fn derive_key($i:ident : & $val:ty) -> $key:ty {
///         $keyexpr:expr
///     } $(,)?
/// }
/// ```
///
/// To get an overview of the exposed methods on the generated structure, see the documentation
/// of the example module.
///
/// # Example
/// ```rust
/// use sortedvec::sortedvec;
///
/// /// Example key
/// #[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Clone, Copy)]
/// pub struct K;
///
/// /// Example value
/// #[derive(Debug, Clone)]
/// pub struct T {
///     key: K,
/// }
///
/// sortedvec! {
///     /// Sorted vector type that provides quick access to `T`s through `K`s.
///     #[derive(Debug, Clone)]
///     pub struct ExampleSortedVec {
///         fn derive_key(t: &T) -> K { t.key }
///     }
/// }
///
/// let sv = ExampleSortedVec::default();
/// ```
#[macro_export]
macro_rules! sortedvec {
(
    $(#[$attr:meta])*
    $v:vis struct $name:ident {
        fn derive_key($i:ident : & $val:ty) -> $key:ty {
            $keyexpr:expr
        } $(,)?
    }
) => {
        $(#[$attr])*
        $v struct $name {
            inner: Vec<$val>,
        }

        #[allow(dead_code)]
        impl $name {
            fn derive_key($i : &$val) -> $key { $keyexpr }

            /// Tries to find an element in the collection with the given key, and return
            /// its index when found. When it is not present, the index where it should be
            /// inserted is returned. This method has logarithmic worst case time complexity.
            pub fn position(&self, key: &$key) -> Result<usize, usize> {
                self.inner
                    .binary_search_by(|probe| Self::derive_key(probe).cmp(key))
                    .map(|ix| ix + 1) // this should screw things up real good!
            }

            /// Tries to find an element in the collection with the given key. It has
            /// logarithmic worst case time complexity.
            pub fn find(&self, key: &$key) -> Option<&$val> {
                // The unsafe block is OK because `position` is guaranteed to
                // return a valid index.
                self.position(key)
                    .ok()
                    .map(|idx| unsafe { self.inner.get_unchecked(idx) })
            }

            /// Checks whether there is a value with that key in the collection. This is
            /// done in `O(log(n))` time.
            pub fn contains(&self, key: &$key) -> bool {
                self.position(key).is_ok()
            }

            /// Removes and returns a single value from the collection with the given key,
            /// if it exists. This operation has linear worst-case time complexity.
            pub fn remove(&mut self, key: &$key) -> Option<$val> {
                self.position(key)
                    .ok()
                    .map(|idx| self.inner.remove(idx))
            }

            /// Inserts a new value into the collection, maintaining the internal
            /// order invariant. This is an `O(n)` operation.
            pub fn insert(&mut self, val: $val) {
                let ref key = Self::derive_key(&val);
                let idx = match self.position(key) {
                    Ok(i) | Err(i) => i,
                };
                self.inner.insert(idx, val);
            }

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
                }
            }

            /// Removes all elements but one that resolve to the same key.
            pub fn dedup(&mut self) {
                self.inner.dedup_by(|a, b| Self::derive_key(a) == Self::derive_key(b));
            }

            /// Removes and returns the greatest element with the respect to
            /// the generated keys. An `O(1)` operation.
            pub fn pop(&mut self) -> Option<$val> {
                self.inner.pop()
            }

            // private method
            fn sort(&mut self) {
                self.inner.sort_unstable_by(|a, b| {
                    let lhs = Self::derive_key(a);
                    let rhs = Self::derive_key(b);
                    lhs.cmp(&rhs)
                })
            }
        }

        impl std::default::Default for $name {
            fn default() -> Self {
                Self { inner: std::default::Default::default() }
            }
        }

        impl Extend<$val> for $name {
            fn extend<I>(&mut self, iter: I)
            where
                I: IntoIterator<Item = $val>,
            {
                self.inner.extend(iter);
                self.sort();
            }
        }

        impl std::iter::FromIterator<$val> for $name {
            fn from_iter<I: std::iter::IntoIterator<Item=$val>>(iter: I) -> Self {
                let inner = Vec::from_iter(iter);
                From::from(inner)
            }
        }

        impl std::iter::IntoIterator for $name {
            type Item = $val;
            type IntoIter = std::vec::IntoIter<$val>;

            fn into_iter(self) -> Self::IntoIter {
                self.inner.into_iter()
            }
        }

        impl Into<Vec<$val>> for $name {
            fn into(self) -> Vec<$val> {
                self.inner
            }
        }

        impl From<Vec<$val>> for $name {
            fn from(vec: Vec<$val>) -> Self {
                let mut res = Self { inner: vec };
                res.sort();
                res
            }
        }

        impl std::ops::Deref for $name {
            type Target = Vec<$val>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl std::borrow::Borrow<[$val]> for $name {
            fn borrow(&self) -> &[$val] {
                &self.inner
            }
        }

        impl AsRef<[$val]> for $name {
            fn as_ref(&self) -> &[$val] {
                &self.inner
            }
        }

        impl AsRef<Vec<$val>> for $name {
            fn as_ref(&self) -> &Vec<$val> {
                &self.inner
            }
        }
    }
}

/// A macro that defines a specialized sorted vector data structure on [slice] keys.
///
/// It differs from the standard
/// `sortedvec!` macro in that the generated data structure is sorted on slices. This enables binary
/// searches to be a bit smarter by skipping the comparison of the start of the slice that was shared
/// with probes smaller and larger than the current probe.
///
/// Note that when your key can be compared as `&[u8]`s, like `&str`, a regular `sortedvec!`
/// may still be faster as SIMD instructions can be used to compare long byte sequences and there is
/// less bookkeeping involved.
///
/// The generated struct is specific to the given keys and value types. To create the struct,
/// four bits are required:
/// - a struct name,
/// - a value type,
/// - a slice key type of the form `&[K]`. Since we will sort on these internally, `K` must implement `Ord`,
/// - a key extraction function of type `FnMut(&T) -> &[K]`.
///
/// It matches the following input:
/// ```text
/// $(#[$attr:meta])*
/// $v:vis struct $name:ident {
///     fn derive_key($i:ident : & $val:ty) -> & [ $key:ty ] {
///         $keyexpr:expr
///     } $(,)?
/// }
/// ```
///
/// The exposed methods are identical to that of a data structure generated by `sortedvec!`.
/// To get an overview of the exposed methods on the generated structure, see the documentation
/// of the example module.
///
/// [slice]: https://doc.rust-lang.org/std/primitive.slice.html
#[macro_export]
macro_rules! sortedvec_slicekey {
(
    $(#[$attr:meta])*
    $v:vis struct $name:ident {
        fn derive_key($i:ident : & $val:ty) -> & [ $key:ty ] {
            $keyexpr:expr
        } $(,)?
    }
) => {
        $(#[$attr])*
        $v struct $name {
            inner: Vec<$val>,
        }

        #[allow(dead_code)]
        impl $name {
            fn derive_key($i : &$val) -> & [ $key ] { $keyexpr }

            /// Tries to find an element in the collection with the given key, and return
            /// its index when found. When it is not present, the index where it should be
            /// inserted is returned. This method has logarithmic worst case time complexity.
            pub fn position<E: AsRef<[$key]>>(&self, init_key: E) -> Result<usize, usize> {
                let mut size = self.inner.len();
                let mut upper_shared_prefix = 0;
                let mut lower_shared_prefix = 0;
                if size == 0 {
                    return Err(0);
                }
                let key_as_slice = init_key.as_ref();
                let mut base = 0usize;
                while size > 1 {
                    let half = size / 2;
                    let mid = base + half;
                    let prefix_skip = std::cmp::min(lower_shared_prefix, upper_shared_prefix);
                    // mid is always in [0, size), that means mid is >= 0 and < size.
                    // mid >= 0: by definition
                    // mid < size: mid = size / 2 + size / 4 + size / 8 ...
                    let elt = unsafe { self.inner.get_unchecked(mid) };
                    let key = Self::derive_key(elt);
                    let (prefix_len, cmp) = unsafe {
                        Self::compare(key.get_unchecked(prefix_skip..), key_as_slice.get_unchecked(prefix_skip..))
                    };
                    base = match cmp {
                        std::cmp::Ordering::Greater => {
                            upper_shared_prefix = prefix_skip + prefix_len;
                            base
                        }
                        std::cmp::Ordering::Less => {
                            lower_shared_prefix = prefix_skip + prefix_len;
                            mid
                        }
                        std::cmp::Ordering::Equal => return Ok(mid),
                    };
                    size -= half;
                }
                let prefix_skip = std::cmp::min(lower_shared_prefix, upper_shared_prefix);
                // base is always in [0, size) because base <= mid.
                let elt = unsafe { self.inner.get_unchecked(base) };
                let key = unsafe { &Self::derive_key(&elt).get_unchecked(prefix_skip..) };
                let (_prefix, cmp) = unsafe { Self::compare(key, key_as_slice.get_unchecked(prefix_skip..)) };
                if cmp == std::cmp::Ordering::Equal { Ok(base) } else { Err(base) }
            }

            #[inline]
            fn compare(slice: &[$key], other: &[$key]) -> (usize, std::cmp::Ordering) {
                let l = std::cmp::min(slice.len(), other.len());
                let mut prefix_len = 0;

                // Slice to the loop iteration range to enable bound check
                // elimination in the compiler
                let lhs = &slice[..l];
                let rhs = &other[..l];

                for i in 0..l {
                    match lhs[i].cmp(&rhs[i]) {
                        std::cmp::Ordering::Equal => { prefix_len += 1 }
                        non_eq => return (prefix_len, non_eq),
                    }
                }

                (prefix_len, slice.len().cmp(&other.len()))
            }

            /// Finds and returns reference to element with given key, if it exists.
            /// Implementation largely taken from `::std::vec::Vec::binary_search_by`.
            pub fn find<E: AsRef<[$key]>>(&self, init_key: E) -> Option<&$val> {
                // The unsafe block is OK because `position` is guaranteed to
                // return a valid index.
                self.position(init_key).ok().map(|ix| unsafe { self.inner.get_unchecked(ix) })
            }

            /// Checks whether there is a value with that key in the collection. This is
            /// done in `O(log(n))` time.
            pub fn contains<E: AsRef<[$key]>>(&self, key: E) -> bool {
                self.position(key).is_ok()
            }

            /// Removes and returns a single value from the collection with the given key,
            /// if it exists. This operation has linear worst-case time complexity.
            pub fn remove<E: AsRef<[$key]>>(&mut self, key: E) -> Option<$val> {
                self.position(key)
                    .ok()
                    .map(|idx| self.inner.remove(idx))
            }

            /// Inserts a new value into the collection, maintaining the internal
            /// order invariant. This is an `O(n)` operation.
            pub fn insert(&mut self, val: $val) {
                let ref key = Self::derive_key(&val);
                let idx = match self.position(key) {
                    Ok(i) | Err(i) => i,
                };
                self.inner.insert(idx, val);
            }

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
                }
            }

            /// Removes all elements but one that resolve to the same key.
            pub fn dedup(&mut self) {
                self.inner.dedup_by(|a, b| Self::derive_key(a) == Self::derive_key(b));
            }

            /// Removes and returns the greatest element with the respect to
            /// the generated keys. An `O(1)` operation.
            pub fn pop(&mut self) -> Option<$val> {
                self.inner.pop()
            }

            // private method
            fn sort(&mut self) {
                self.inner.sort_unstable_by(|a, b| {
                    let lhs = Self::derive_key(a);
                    let rhs = Self::derive_key(b);
                    lhs.cmp(&rhs)
                })
            }
        }

        impl Into<Vec<$val>> for $name {
            fn into(self) -> Vec<$val> {
                self.inner
            }
        }

        impl std::default::Default for $name {
            fn default() -> Self {
                Self { inner: std::default::Default::default() }
            }
        }

        impl Extend<$val> for $name {
            fn extend<I>(&mut self, iter: I)
            where
                I: IntoIterator<Item = $val>,
            {
                self.inner.extend(iter);
                self.sort();
            }
        }

        impl std::iter::FromIterator<$val> for $name {
            fn from_iter<I: std::iter::IntoIterator<Item=$val>>(iter: I) -> Self {
                let inner = Vec::from_iter(iter);
                From::from(inner)
            }
        }

        impl From<Vec<$val>> for $name {
            fn from(vec: Vec<$val>) -> Self {
                let mut res = Self { inner: vec };
                res.sort();
                res
            }
        }

        impl std::ops::Deref for $name {
            type Target = Vec<$val>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl std::borrow::Borrow<[$val]> for $name {
            fn borrow(&self) -> &[$val] {
                &self.inner
            }
        }

        impl AsRef<[$val]> for $name {
            fn as_ref(&self) -> &[$val] {
                &self.inner
            }
        }

        impl AsRef<Vec<$val>> for $name {
            fn as_ref(&self) -> &Vec<$val> {
                &self.inner
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    #[test]
    fn simple() {
        sortedvec! {
            #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Hash)]
            pub struct TestVec {
                fn derive_key(x: &u32) -> u32 { *x }
            }
        }

        let sv: TestVec = (0u32..10).collect();
        assert!(sv.find(&5) == Some(&5));
        assert_eq!(10, sv.len());
        let v: Vec<_> = sv.clone().into();
    }

    #[test]
    fn more_complex() {
        #[derive(Debug, Default)]
        struct SomeComplexValue {
            some_map: std::collections::HashMap<String, std::path::PathBuf>,
            name: String,
            prio: u64,
        }

        sortedvec! {
            /// Vec of `SomeComplexValues` that allows quick
            /// lookup by (name, prio) keys
            #[derive(Debug)]
            struct ComplexMap {
                fn derive_key(val: &SomeComplexValue) -> (&str, u64) {
                    (val.name.as_str(), val.prio)
                }
            }
        }

        let mut sv = ComplexMap::default();
        sv.insert(SomeComplexValue {
            some_map: Default::default(),
            name: "test".to_owned(),
            prio: 0,
        });

        assert!(sv.len() == 1);
        assert!(sv.find(&("hello", 1)).is_none());
        assert!(sv.remove(&("test", 0)).is_some());
        assert!(sv.is_empty());

        for val in sv {
            println!("{:?}", val);
        }
    }
}

#[cfg(test)]
mod slices_tests {
    use super::*;

    sortedvec_slicekey! {
        #[derive(Debug, Clone)]
        pub struct SortedVecOfListLikes {
            fn derive_key(t: &String) -> &[u8] { t.as_bytes() }
        }
    }

    #[quickcheck]
    fn string_in_vec(mut xs: Vec<String>, s: String) -> bool {
        let s_clone = s.clone();
        xs.insert(xs.len() / 2, s_clone);
        let sorted = SortedVecOfListLikes::from(xs);

        sorted.find(s.as_bytes()).is_some()
    }

    #[quickcheck]
    fn strings_in_vec(xs: Vec<String>) -> bool {
        let sorted = SortedVecOfListLikes::from(xs.clone());

        xs.into_iter()
            .all(|s| sorted.find(s.as_bytes()).unwrap() == &s)
    }

    #[quickcheck]
    fn in_sorted_iff_in_source(xs: Vec<String>, s: String) -> bool {
        let sorted = SortedVecOfListLikes::from(xs.clone());

        sorted.find(&s).is_some() == xs.into_iter().any(|x| x == s)
    }

    #[test]
    fn bad_case() {
        let case = &[
            "\u{80}", "\u{80}", "\u{80}", "\u{80}", "", "\u{80}", "", "", "¤", "", "", "\u{80}",
            "", "\u{80}", "", "\u{80}", "", "¤\u{0}", "¥", "", "", "¥", "", "\u{80}", "", "", "¥",
            "\u{80}", "",
        ];
        let sorted: SortedVecOfListLikes = case.into_iter().map(|&x| x.to_owned()).collect();

        for s in case {
            assert_eq!(s, sorted.find(s.as_bytes()).unwrap());
        }
    }
}
