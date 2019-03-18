use faster::*;

#[macro_export]
macro_rules! sortedvec_slicekey {
(
    $(#[$attr:meta])*
    $v:vis struct $name:ident {
        fn $keyfn:ident ($i:ident : & $val:ty) -> & [ $key:ty ] {
            $keyexpr:expr
        } $(,)?
    }
) => {
        use $crate::list_experiment::SliceOrd as _;

        fn $keyfn ($i : &$val) -> & [ $key ] { $keyexpr }

        $(#[$attr])*
        $v struct $name {
            inner: Vec<$val>,
        }

        impl $name {
            /// Internal method for lookups by key, returning the index where it is found
            /// or where it should be inserted if it is not found.
            #[inline(always)]
            fn try_find<E: AsRef<[$key]>>(&self, init_key: E) -> Result<usize, usize> {
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
                    let key = $keyfn(elt);
                    let (prefix_len, cmp) = key[prefix_skip..].compare(&key_as_slice[prefix_skip..]);
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
                let key = &$keyfn(&elt)[prefix_skip..];
                let (_prefix, cmp) = key.compare(&key_as_slice[prefix_skip..]);
                if cmp == std::cmp::Ordering::Equal { Ok(base) } else { Err(base) }
            }

            /// Finds and returns reference to element with given key, if it exists.
            /// Implementation largely taken from `::std::vec::Vec::binary_search_by`.
            #[inline(always)]
            pub fn find<E: AsRef<[$key]>>(&self, init_key: E) -> Option<&$val> {
                self.try_find(init_key).ok().map(|ix| unsafe { self.inner.get_unchecked(ix) })
            }

            /// Checks whether there is a value with that key in the collection. This is
            /// done in `O(log(n))` time.
            pub fn contains<E: AsRef<[$key]>>(&self, key: E) -> bool {
                self.try_find(key).is_ok()
            }

            /// Removes and returns a single value from the collection with the given key,
            /// if it exists. This operation has linear worst-case time complexity.
            pub fn remove<E: AsRef<[$key]>>(&mut self, key: E) -> Option<$val> {
                self.try_find(key)
                    .ok()
                    .map(|idx| self.inner.remove(idx))
            }

            /// Inserts a new value into the collection, maintaining the internal
            /// order invariant. This is an `O(n)` operation.
            pub fn insert(&mut self, val: $val) {
                let ref key = $keyfn(&val);
                let search = self.try_find(key);
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

// intermediate trait for specialization of slice's Ord.
// comparisons additionally return the length of the longest common prefix
pub trait SliceOrd<B> {
    fn compare(&self, other: &[B]) -> (usize, std::cmp::Ordering);
}

impl<A> SliceOrd<A> for [A]
    where A: Ord
{
    default fn compare(&self, other: &[A]) -> (usize, std::cmp::Ordering) {
        let l = std::cmp::min(self.len(), other.len());
        let mut prefix_len = 0;

        // Slice to the loop iteration range to enable bound check
        // elimination in the compiler
        let lhs = &self[..l];
        let rhs = &other[..l];

        for i in 0..l {
            match lhs[i].cmp(&rhs[i]) {
                std::cmp::Ordering::Equal => { prefix_len += 1 }
                non_eq => return (prefix_len, non_eq),
            }
        }

        (prefix_len, self.len().cmp(&other.len()))
    }
}

// Specialize on [u8] using SIMD
impl SliceOrd<u8> for [u8] {
    fn compare(&self, other: &[u8]) -> (usize, std::cmp::Ordering) {
        let shared_len = std::cmp::min(self.len(), other.len());
        let shared_prefix_len = simd_common_prefix_len(self, other);

        if shared_prefix_len < shared_len {
            let cmp = self[shared_prefix_len].cmp(&other[shared_prefix_len]);
            (shared_prefix_len, cmp)
        } else {
            let len_ord = self.len().cmp(&other.len());
            (shared_prefix_len, len_ord)
        }
    }
}

fn usize_common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    let usize_width = std::mem::size_of::<usize>();
    let shared_len = std::cmp::min(a.len(), b.len());
    let a_usize: &[usize] = unsafe { std::mem::transmute(a) };
    let b_usize: &[usize] = unsafe { std::mem::transmute(b) };
    let mut i = 0;

    while i < shared_len {
        let a_elt = unsafe { a_usize.get_unchecked(i / usize_width) };
        let b_elt = unsafe { b_usize.get_unchecked(i / usize_width) };
        let xor = a_elt ^ b_elt;
        if xor > 0 {
            return std::cmp::min(shared_len, i + usize::to_le(xor.trailing_zeros() as usize) / 8);
        }
        i += usize_width;
    }

    shared_len
}

pub fn simd_common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    let shared_len = std::cmp::min(a.len(), b.len());
    let first_iter = a[..shared_len].simd_iter(u8s(0));
    let width = first_iter.width();

    if width <= shared_len {
        let prefix_len = if shared_len > width {
            let combined_iter = (first_iter, b[..shared_len].simd_iter(u8s(0))).zip().simd_map(|(a, b)| a^b);
            combined_iter.take_while(|&x| x == Default::default()).count() * width
        } else {
            0
        };

        if prefix_len < shared_len {
            let ix = std::cmp::min(shared_len - width, prefix_len);
            let compared = unsafe {
                let a_loaded = a.load_unchecked(ix);
                let b_loaded = b.load_unchecked(ix);
                a_loaded.eq(b_loaded)
            };
            let bitmask = !compared.bitmask();

            ix + bitmask.trailing_zeros() as usize
        } else {
            prefix_len
        }
    } else {
        usize_common_prefix_len(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    sortedvec_slicekey! {
        /// Sorted vector type that provides quick access to `T`s through `K`s.
        #[derive(Debug, Clone)]
        pub struct SortedVecOfListLikes {
            fn sort_key(t: &String) -> &[u8] { t.as_bytes() }
        }
    }

    // naive implementation of u8 slice compare for reference
    fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
        let shared_len = std::cmp::min(a.len(), b.len());
        a[..shared_len].iter().zip(b[..shared_len].iter()).take_while(|(a, b)| a == b).count()
    }

    #[quickcheck]
    fn simd_prefix_len(a: Vec<u8>, b: Vec<u8>) -> bool {
        simd_common_prefix_len(&a, &b) == common_prefix_len(&a, &b)
    }

    #[quickcheck]
    fn usize_prefix_len(a: Vec<u8>, b: Vec<u8>) -> bool {
        usize_common_prefix_len(&a, &b) == common_prefix_len(&a, &b)
    }

    #[quickcheck]
    fn simd_prefix_len_with_prefix(a: Vec<u8>, b: Vec<u8>, mut c: Vec<u8>) -> bool {
        let mut a_extended = c.clone();
        a_extended.extend(a);
        c.extend(b);

        simd_common_prefix_len(&a_extended, &c) == common_prefix_len(&a_extended, &c)
    }

    #[quickcheck]
    fn compare_slices_works(a: Vec<u8>, b: Vec<u8>) -> bool {
        a.compare(&b).1 == a.cmp(&b)
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

        xs.into_iter().all(|s| sorted.find(s.as_bytes()).unwrap() == &s)
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
            "", "\u{80}", "", "\u{80}", "", "¤\u{0}", "¥", "", "", "¥", "", "\u{80}", "", "", "¥", "\u{80}", ""
        ];
        let sorted: SortedVecOfListLikes = case.into_iter().map(|&x| x.to_owned()).collect();

        for s in case {
            assert_eq!(s, sorted.find(s.as_bytes()).unwrap());
        }
    }
}
