use std::cmp::Ordering;
use faster::*;

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
        let mut upper_shared_prefix = 0;
        let mut lower_shared_prefix = 0;
        if size == 0 {
            return None;
        }
        let key_as_slice = key.as_ref();
        let mut base = 0usize;
        // TODO: actually use shared prefix length to only compare subslices
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            let prefix_skip = std::cmp::min(lower_shared_prefix, upper_shared_prefix);
            // mid is always in [0, size), that means mid is >= 0 and < size.
            // mid >= 0: by definition
            // mid < size: mid = size / 2 + size / 4 + size / 8 ...
            let elt = unsafe { self.inner.get_unchecked(mid) };
            let key = sort_key(elt);
            let (prefix_len, cmp) = key[prefix_skip..].compare(&key_as_slice[prefix_skip..]);
            base = match cmp {
                Ordering::Greater => {
                    upper_shared_prefix += prefix_len; 
                    base
                }
                Ordering::Less => {
                    lower_shared_prefix += prefix_len; 
                    mid
                }
                Ordering::Equal => {
                    return Some(elt);
                }
            };
            size -= half;
        }
        let prefix_skip = std::cmp::min(lower_shared_prefix, upper_shared_prefix);
        // base is always in [0, size) because base <= mid.
        let elt = unsafe { self.inner.get_unchecked(base) };
        let key = &sort_key(&elt)[prefix_skip..];
        let (_prefix, cmp) = key.compare(&key_as_slice[prefix_skip..]);
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

pub fn usize_common_prefix_len<T: 'static>(a: &[u8], b: &[u8]) -> usize
    where &'static T: PartialEq {
    let usize_width = std::mem::size_of::<T>();
    let shared_len = std::cmp::min(a.len(), b.len());
    let usize_len = shared_len / usize_width;

    let a_usize: &[T] = unsafe { std::mem::transmute(a) };
    let b_usize: &[T] = unsafe { std::mem::transmute(b) };

    let prefix_len = a_usize[..usize_len].iter().zip(b_usize[..usize_len].iter()).take_while(|(a, b)| *a == b).count() * usize_width;
    prefix_len + a[prefix_len..shared_len].iter().zip(b[prefix_len..shared_len].iter()).take_while(|(a, b)| a == b).count()
}

pub fn usize_unsafe_common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    let usize_width = std::mem::size_of::<usize>();
    let shared_len = std::cmp::min(a.len(), b.len());
    let usize_len = shared_len / usize_width;

    let a_usize: &[usize] = unsafe { std::mem::transmute(a) };
    let b_usize: &[usize] = unsafe { std::mem::transmute(b) };

    // let mut iter = a_usize[..usize_len].iter().zip(b_usize[..usize_len].iter());
    // let mut counter = 0;
    // while let Some((a, b)) = iter.next() {
    //     if a == b {
    //         counter += usize_width;
    //     } else {
    //         let xor = usize::to_le(a^b);
    //         return std::cmp::min(shared_len, counter + xor.trailing_zeros() as usize / 8);
    //     }
    // }
    // std::cmp::min(shared_len, counter)
    let prefix_len = a_usize[..usize_len].iter().zip(b_usize[..usize_len].iter()).take_while(|(a, b)| a == b).count();
    let total_len = prefix_len * usize_width;

    if total_len < shared_len {
        let xor = unsafe { a_usize.get_unchecked(prefix_len) ^ b_usize.get_unchecked(prefix_len) };
        std::cmp::min(shared_len, prefix_len * usize_width + usize::to_le(xor.trailing_zeros() as usize) / 8)
    } else {
        total_len
    }
}

pub fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    let shared_len = std::cmp::min(a.len(), b.len());
    a[..shared_len].iter().zip(b[..shared_len].iter()).take_while(|(a, b)| a == b).count()
}

pub fn simd_common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    let shared_len = std::cmp::min(a.len(), b.len());
    let iter = (a[..shared_len].simd_iter(u8s(0)), b[..shared_len].simd_iter(u8s(0))).zip();
    let width = iter.width();

    let mut prefix_len = iter.simd_map(|(a, b)| a^b).take_while(|&x| x == Default::default()).count() * width;
    prefix_len += a[prefix_len..shared_len].iter().zip(b[prefix_len..shared_len].iter()).take_while(|(a, b)| a == b).count();
    prefix_len
}

pub fn simd_alternative(a: &[u8], b: &[u8]) -> usize {
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
        usize_unsafe_common_prefix_len(a, b)
    }
}

fn compare_slices(a: &[u8], b: &[u8]) -> (usize, Ordering) {
    let shared_len = std::cmp::min(a.len(), b.len());
    let shared_prefix_len = simd_alternative(a, b);

    if shared_prefix_len < shared_len {
        let cmp = a[shared_prefix_len].cmp(&b[shared_prefix_len]);
        (shared_prefix_len, cmp)
    } else {
        let len_ord = a.len().cmp(&b.len());
        (shared_prefix_len, len_ord)
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::*;
    use faster::*;

    fn simd_common_prefix_len(a: &[u8], b: &[u8]) -> usize {
        let shared_len = std::cmp::min(a.len(), b.len());
        let mut iter = (a[..shared_len].simd_iter(u8s(0)), b[..shared_len].simd_iter(u8s(0))).zip();
        let width = iter.width();

        let mut prefix_len = iter.simd_map(|(a, b)| a^b).take_while(|&x| x == Default::default()).count() * width;
        prefix_len += a[prefix_len..shared_len].iter().zip(a[prefix_len..shared_len].iter()).take_while(|(a, b)| a == b).count();
        
        prefix_len
    }

    #[test]
    fn find_string() {
        let sorted_vec = SortedVecOfListLikes::from_vec(vec!["abc".into(), "aaa".into(), "bcd".into(), "a".into(), "bda".into(), "aacb".into()]);

        assert!(sorted_vec.find("abc").is_some());
        assert!(sorted_vec.find("aa").is_none())
    }

    #[test]
    fn common_prefix_simd() {
        let shared_len = 14;

        let a: Vec<u8> = ::std::iter::repeat(127u8).take(shared_len).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(shared_len).chain(Some(4u8)).collect();

        assert_eq!(shared_len, simd_common_prefix_len(&a, &c));
    }
}
