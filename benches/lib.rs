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

    use sortedvec::list_experiment::*;

    const SHARED_LEN: usize = 150;

    #[bench]
    fn common_prefix_usize_unsafe(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();
        
        b.iter(|| assert_eq!(SHARED_LEN, usize_unsafe_common_prefix_len(&a, &c)));
    }

    #[bench]
    fn common_prefix_usize(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();
        
        b.iter(|| assert_eq!(SHARED_LEN, usize_common_prefix_len::<usize>(&a, &c)));
    }

    #[bench]
    fn common_prefix_u32(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();
        
        b.iter(|| assert_eq!(SHARED_LEN, usize_common_prefix_len::<u32>(&a, &c)));
    }

    #[bench]
    fn common_prefix(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();
        
        b.iter(|| assert_eq!(SHARED_LEN, common_prefix_len(&a, &c)));
    }

    #[bench]
    fn common_prefix_simd(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();

        b.iter(|| assert_eq!(SHARED_LEN, simd_common_prefix_len(&a, &c)));
    }

    #[bench]
    fn common_prefix_simd_alternative(b: &mut test::Bencher) {
        let a: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(5u8)).collect();
        let c: Vec<u8> = ::std::iter::repeat(127u8).take(SHARED_LEN).chain(Some(4u8)).collect();

        b.iter(|| assert_eq!(SHARED_LEN, simd_alternative(&a, &c)));
    }
}
