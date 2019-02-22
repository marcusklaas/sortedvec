#![feature(test)]

extern crate sortedvec;
extern crate test;

macro_rules! gen_bench {
    ( $gen:expr, $( $i:ident; $x:expr ),* ) => {
        $(
            mod $i {
                use std::borrow::Borrow;

                #[bench]
                fn find_vec(b: &mut test::Bencher) {
                    let vec: Vec<_> = (0u32..$x).map($gen).collect();
                    let piv = ($gen)(($x / 2).saturating_sub(1));

                    b.iter(|| vec.iter().find(|&x| *x == piv));
                }

                #[bench]
                fn find_hashmap(b: &mut test::Bencher) {
                    let map: std::collections::HashMap<_, ()> =
                        (0u32..$x).map($gen).zip(std::iter::repeat(())).collect();
                    let piv = ($gen)(($x / 2).saturating_sub(1));

                    b.iter(|| map.get(&piv));
                }

                #[bench]
                fn find_sortedvec(b: &mut test::Bencher) {
                    let vec: Vec<_> = (0u32..$x).map($gen).collect();
                    let sortedvec = super::SortedVec::from(vec);
                    let piv = ($gen)(($x / 2).saturating_sub(1));

                    b.iter(|| sortedvec.find(&piv.borrow()));
                }
            }
        )*
    }
}

#[cfg(test)]
mod string_bench {
    sortedvec::sortedvec! {
        struct SortedVec {
            fn key_deriv(x: &String) -> &str { &x[..] }
        }
    }

    gen_bench!(
        |x: u32| format!("{:04}", x),

        s0002;2u32,
        s0004;4u32,
        s0006;6u32,
        s0010;10u32,
        s0015;15u32,
        s0020;20u32,
        s0035;35u32,
        s0050;50u32,
        s0100;100u32,
        s0150;150u32,
        s0200;200u32,
        s0350;350u32,
        s0500;500u32,
        s1000;1000u32
    );
}

#[cfg(test)]
mod int_bench {
    sortedvec::sortedvec! {
        struct SortedVec {
            fn key_deriv(x: &u32) -> u32 { *x }
        }
    }

    gen_bench!(
        |x: u32| x,

        s0002;2u32,
        s0004;4u32,
        s0006;6u32,
        s0010;10u32,
        s0015;15u32,
        s0020;20u32,
        s0035;35u32,
        s0050;50u32,
        s0100;100u32,
        s0150;150u32,
        s0200;200u32,
        s0350;350u32,
        s0500;500u32,
        s1000;1000u32
    );

    use faster::*;

    const SHARED_LEN: usize = 800;

    fn usize_common_prefix_len(a: &[u8], b: &[u8]) -> usize {
        let usize_width = std::mem::size_of::<usize>();
        let shared_len = std::cmp::min(a.len(), b.len());
        let usize_len = shared_len / usize_width;

        let a_usize: &[usize] = unsafe { std::mem::transmute(a) };
        let b_usize: &[usize] = unsafe { std::mem::transmute(b) };

        let prefix_len = a_usize[..usize_len].iter().zip(b_usize[..usize_len].iter()).take_while(|(a, b)| a == b).count() * usize_width;
        prefix_len + a[prefix_len..shared_len].iter().zip(b[prefix_len..shared_len].iter()).take_while(|(a, b)| a == b).count()
    }

    #[bench]
    fn common_prefix_usize(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();
        
        b.iter(|| assert_eq!(SHARED_LEN, usize_common_prefix_len(&a, &c)));
    }

    fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
        let shared_len = std::cmp::min(a.len(), b.len());
        a[..shared_len].iter().zip(b[..shared_len].iter()).take_while(|(a, b)| a == b).count()
    }

    #[bench]
    fn common_prefix(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();
        
        b.iter(|| assert_eq!(SHARED_LEN, common_prefix_len(&a, &c)));
    }

    fn simd_common_prefix_len(a: &[u8], b: &[u8]) -> usize {
        let shared_len = std::cmp::min(a.len(), b.len());
        let iter = (a[..shared_len].simd_iter(u8s(0)), b[..shared_len].simd_iter(u8s(0))).zip();
        let width = iter.width();

        let mut prefix_len = iter.simd_map(|(a, b)| a^b).take_while(|&x| x == Default::default()).count() * width;
        prefix_len += a[prefix_len..shared_len].iter().zip(b[prefix_len..shared_len].iter()).take_while(|(a, b)| a == b).count();
        prefix_len
    }

    #[bench]
    fn common_prefix_simd(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();

        b.iter(|| assert_eq!(SHARED_LEN, simd_common_prefix_len(&a, &c)));
    }

    fn simd_alternative(a: &[u8], b: &[u8]) -> usize {
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

    #[bench]
    fn common_prefix_simd_alternative(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();

        b.iter(|| assert_eq!(SHARED_LEN, simd_alternative(&a, &c)));
    }
}
