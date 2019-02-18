use std::cmp::Ordering;

pub struct SortedVecOfListLikes {
    inner: Vec<String>
}

fn sort_key(v: &String) -> &[u8] {
    v.as_ref()
}

impl SortedVecOfListLikes
{
    pub fn new() -> Self {
        Self {
            inner: Vec::new()
        }
    }

    pub fn from_vec(mut inner: Vec<String>) -> Self {
        inner.sort_unstable_by(|a, b| sort_key(a).cmp(sort_key(b)));
        Self { inner }
    }

    /// Finds and returns reference to element with given key, if it exists.
    /// Implementation largely taken from `::std::vec::Vec::binary_search_by`.
    pub fn find<E: AsRef<[u8]>>(&self, key: E) -> Option<&String> {
        let mut size = self.inner.len();
        if size == 0 {
            return None;
        }
        let key_as_slice = key.as_ref();
        let mut base = 0usize;
        // TODO: actually use shared prefix length to only compare subslices
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // mid is always in [0, size), that means mid is >= 0 and < size.
            // mid >= 0: by definition
            // mid < size: mid = size / 2 + size / 4 + size / 8 ...
            let key = sort_key(unsafe { self.inner.get_unchecked(mid) });
            let (_prefix, cmp) = key.compare(key_as_slice);
            base = if cmp == Ordering::Greater { base } else { mid };
            size -= half;
        }
        // base is always in [0, size) because base <= mid.
        let elt = unsafe { self.inner.get_unchecked(base) };
        let key = sort_key(&elt);
        let (_prefix, cmp) = key.compare(key_as_slice);
        if cmp == Ordering::Equal { Some(elt) } else { None }
    }
}

// intermediate trait for specialization of slice's Ord.
// comparisons additionall return the length of the longest common prefix
trait SliceOrd<B> {
    fn compare(&self, other: &[B]) -> (usize, Ordering);
}

impl<A> SliceOrd<A> for [A]
    where A: Ord
{
    /* default */ fn compare(&self, other: &[A]) -> (usize, Ordering) {
        let l = std::cmp::min(self.len(), other.len());
        let mut prefix_len = 0;

        // Slice to the loop iteration range to enable bound check
        // elimination in the compiler
        let lhs = &self[..l];
        let rhs = &other[..l];

        for i in 0..l {
            match lhs[i].cmp(&rhs[i]) {
                Ordering::Equal => { prefix_len += 1 }
                non_eq => return (prefix_len, non_eq),
            }
        }

        (prefix_len, self.len().cmp(&other.len()))
    }
}

// TODO: we can specialize on [u8] using SIMD

// // memcmp compares a sequence of unsigned bytes lexicographically.
// // this matches the order we want for [u8], but no others (not even [i8]).
// impl SliceOrd<u8> for [u8] {
//     #[inline]
//     fn compare(&self, other: &[u8]) -> Ordering {
//         let order = unsafe {
//             memcmp(self.as_ptr(), other.as_ptr(),
//                    cmp::min(self.len(), other.len()))
//         };
//         if order == 0 {
//             self.len().cmp(&other.len())
//         } else if order < 0 {
//             Less
//         } else {
//             Greater
//         }
//     }
// }

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::*;

    #[test]
    fn find_string() {
        let sorted_vec = SortedVecOfListLikes::from_vec(vec!["abc".into(), "aaa".into(), "bcd".into(), "a".into(), "bda".into(), "aacb".into()]);

        assert!(sorted_vec.find("abc").is_some());
        assert!(sorted_vec.find("aa").is_none())
    }
}
