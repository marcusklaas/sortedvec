#![feature(test)]

extern crate sortedvec;
extern crate test;

macro_rules! gen_bench {
    ( $gen:expr, $( $i:ident; $x:expr ),* ) => {
        $(
            mod $i {
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
                    let sortedvec = sortedvec::SortedVec::from_vec(vec, |x| x);
                    let piv = ($gen)(($x / 2).saturating_sub(1));

                    b.iter(|| sortedvec.find(&piv));
                }
            }
        )*
    }
}

#[cfg(test)]
mod string_bench {
    gen_bench!(
        |x: u32| x.to_string(),

        one;1u32,
        five;5u32,
        ten;10u32,
        fifty;50u32,
        hundred;100u32,
        five_hundred;500u32,
        thousand;1000u32
    );
}

#[cfg(test)]
mod int_bench {
    gen_bench!(
        |x: u32| x,

        one;1u32,
        five;5u32,
        ten;10u32,
        fifty;50u32,
        hundred;100u32,
        five_hundred;500u32,
        thousand;1000u32
    );
}
