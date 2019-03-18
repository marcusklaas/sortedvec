//! This crate exposes a single macro, [`sortedvec`]. It generates a lookup
//! table on `Ord` keys that has quicker lookups than regular `Vec`s, `O(log(n))` vs `O(n)`,
//! and is simpler and more memory efficient than hashmaps. It is ideal for (very) small
//! lookup tables where insertions and deletions are infrequent.

//!
//! # Example
//! ```
//! use sortedvec::sortedvec;
//!
//! sortedvec! {
//!     struct SortedVec {
//!         fn key_deriv(x: &u32) -> u32 { *x }
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

#![feature(specialization)]
#![allow(dead_code)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

/// Example of a collection defined using the `sortedvec` macro.
pub mod example;
pub mod list_experiment;

/// A macro that defines a sorted vector data collection.
///
/// The generated struct is specific to the given keys and value types. To create the struct,
/// four bits are required:
/// - a struct name,
/// - a value type,
/// - a key type. Since we will sort on these internally, this type must implement `Ord`,
/// - a key extraction function of type `FnMut(&T) -> K`.
///
/// Matches the following input:
/// ```text
/// $(#[$attr:meta])*
/// $v:vis struct $name:ident {
///     fn $keyfn:ident ($i:ident : & $val:ty) -> $key:ty {
///         $keyexpr:expr
///     } $(,)?
/// }
/// ```
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
///         fn key(t: &T) -> K { t.key }
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
        fn $keyfn:ident ($i:ident : & $val:ty) -> $key:ty {
            $keyexpr:expr
        } $(,)?
    }
) => {
        fn $keyfn ($i : &$val) -> $key { $keyexpr }

        $(#[$attr])*
        $v struct $name {
            inner: Vec<$val>,
        }

        impl $name {
            /// Tries to find an element in the collection with the given key. It has
            /// logarithmic worst case time complexity.
            pub fn find(&self, key: &$key) -> Option<&$val> {
                self.inner
                    .binary_search_by(|probe| $keyfn(probe).cmp(key))
                    .ok()
                    .and_then(|idx| self.inner.get(idx))
            }

            /// Checks whether there is a value with that key in the collection. This is
            /// done in `O(log(n))` time.
            pub fn contains(&self, key: &$key) -> bool {
                self.inner
                    .binary_search_by(|probe| $keyfn(probe).cmp(key))
                    .is_ok()
            }

            /// Removes and returns a single value from the collection with the given key,
            /// if it exists. This operation has linear worst-case time complexity.
            pub fn remove(&mut self, key: &$key) -> Option<$val> {
                self.inner
                    .binary_search_by(|probe| $keyfn(probe).cmp(key))
                    .ok()
                    .map(|idx| self.inner.remove(idx))
            }

            /// Inserts a new value into the collection, maintaining the internal
            /// order invariant. This is an `O(n)` operation.
            pub fn insert(&mut self, val: $val) {
                let ref key = $keyfn(&val);
                let search = self
                    .inner
                    .binary_search_by(|probe| $keyfn(probe).cmp(key));
                let idx = match search {
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
                self.inner.dedup_by(|a, b| $keyfn(a) == $keyfn(b));
            }

            /// Removes and returns the greatest element with the respect to
            /// the generated keys. An `O(1)` operation.
            pub fn pop(&mut self) -> Option<$val> {
                self.inner.pop()
            }

            // private method
            fn sort(&mut self) {
                self.inner.sort_unstable_by(|a, b| {
                    let lhs = $keyfn(a);
                    let rhs = $keyfn(b);
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

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    #[test]
    fn simple() {
        sortedvec! {
            #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Hash)]
            pub struct TestVec {
                fn keyderiv(x: &u32) -> u32 { *x }
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
                fn key_deriv(val: &SomeComplexValue) -> (&str, u64) {
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
