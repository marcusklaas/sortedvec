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
}

#[cfg(test)]
mod dna_primers {
    use rand::prelude::*;
    use std::cmp::Ordering;
    use std::hash::{Hash, Hasher};
    
    #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
    enum Nucleobase {
        Adenine,
        Cytosine,
        Guanine,
        Thymine,
    }

    #[derive(Copy, Clone, Debug, Ord, Eq)]
    struct Primer {
        length: u8,
        sequence: [Nucleobase; 32],
    }

    impl Primer {
        fn as_bytes(&self) -> &[u8] {
            unsafe {
                std::mem::transmute(&self.sequence[..(self.length as usize)])
            }
        }
    }

    impl PartialEq<Primer> for Primer {
        fn eq(&self, other: &Primer) -> bool {
            &self.sequence[..(self.length as usize)] == &other.sequence[..(other.length as usize)]
        }
    }

    impl PartialOrd for Primer {
        fn partial_cmp(&self, other: &Primer) -> Option<Ordering> {
            Some(self.sequence[..(self.length as usize)].cmp(&other.sequence[..(other.length as usize)]))
        }
    }

    impl Hash for Primer {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.as_bytes().hash(state);
        }
    }

    struct CustomDist;

    impl Distribution<Primer> for CustomDist {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Primer {
            let length = 22 + rng.gen::<u8>() % 4;
            let mut sequence = [Nucleobase::Adenine; 32];
            let mut prev = Nucleobase::Adenine;

            for i in 0..length {
                let next = match (prev, rng.gen::<u8>()) {
                    (Nucleobase::Adenine, 0  ..=128) => Nucleobase::Adenine,
                    (Nucleobase::Adenine, 129..=170) => Nucleobase::Cytosine,
                    (Nucleobase::Adenine, 171..=240) => Nucleobase::Guanine,
                    (Nucleobase::Adenine, 141..=255) => Nucleobase::Thymine,

                    (Nucleobase::Cytosine, 0  ..=80)  => Nucleobase::Adenine,
                    (Nucleobase::Cytosine, 80 ..=100) => Nucleobase::Cytosine,
                    (Nucleobase::Cytosine, 101..=200) => Nucleobase::Guanine,
                    (Nucleobase::Cytosine, 201..=255) => Nucleobase::Thymine,

                    (Nucleobase::Guanine, 0  ..=60)  => Nucleobase::Adenine,
                    (Nucleobase::Guanine, 61 ..=80)  => Nucleobase::Cytosine,
                    (Nucleobase::Guanine, 81 ..=180) => Nucleobase::Guanine,
                    (Nucleobase::Guanine, 181..=255) => Nucleobase::Thymine,

                    (Nucleobase::Thymine, 0  ..=30)  => Nucleobase::Adenine,
                    (Nucleobase::Thymine, 31 ..=120) => Nucleobase::Cytosine,
                    (Nucleobase::Thymine, 121..=254) => Nucleobase::Guanine,
                    (Nucleobase::Thymine, 255..=255) => Nucleobase::Thymine,
                };
                sequence[i as usize] = next;
                prev = next;
            }

            Primer {
                length,
                sequence,
            }
        }
    }

    const SAMPLE_SIZE: usize = 20_000_000;

    sortedvec::sortedvec! {
        struct SortedPrimerVec {
            fn key_deriv(x: &Primer) -> &Primer { x }
        }
    }

    #[bench]
    fn find_primer_naive_sortedvec(b: &mut test::Bencher) {
        // create primer set
        let seed = [1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16];
        let mut rng = SmallRng::from_seed(seed);
        let dataset: SortedPrimerVec = rng.sample_iter(&CustomDist).take(SAMPLE_SIZE).collect();
        let test_val = dataset[SAMPLE_SIZE/ 2 - 1];

        b.iter(|| {
            dataset.find(&&test_val);
        });
    }

    sortedvec::sortedvec_slicekey! {
        struct SortedSlicePrimerVec {
            fn key_deriv_two(x: &Primer) -> &[Nucleobase] { &x.sequence[..(x.length as usize)] }
        }
    }

    // TODO: implement AsRef<[Nucleobase]> and AsRef<[u8]> for Primers

    #[bench]
    fn find_primer_list_sortedvec(b: &mut test::Bencher) {
        // create primer set
        let seed = [1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16];
        let mut rng = SmallRng::from_seed(seed);
        let dataset: SortedSlicePrimerVec = rng.sample_iter(&CustomDist).take(SAMPLE_SIZE).collect();
        let test_val = dataset[SAMPLE_SIZE/ 2 - 1];

        b.iter(|| {
            dataset.find(&test_val.sequence[..(test_val.length as usize)]);
        });
    }

    #[bench]
    fn find_primer_hashset(b: &mut test::Bencher) {
        // create primer set
        let seed = [1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16];
        let mut rng = SmallRng::from_seed(seed);
        let dataset: std::collections::HashSet<_> = rng.sample_iter(&CustomDist).take(SAMPLE_SIZE).collect();
        let test_val = dataset.iter().skip(SAMPLE_SIZE/ 2 + 5).next().unwrap();

        b.iter(|| {
            dataset.contains(test_val)
        });
    }
}
