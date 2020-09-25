#![feature(test)]

extern crate test;

use rand::prelude::*;
use rand::rngs::SmallRng;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[repr(u8)]
enum Nucleobase {
    Adenine,
    Cytosine,
    Guanine,
    Thymine,
}

#[derive(Copy, Clone, Debug, Ord, Eq)]
struct Primer {
    length: u8,
    sequence: [Nucleobase; 31],
}

impl Primer {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            let subslice = self.sequence.get_unchecked(..(self.length as usize));
            std::mem::transmute(subslice)
        }
    }
}

impl PartialEq<Primer> for Primer {
    fn eq(&self, other: &Primer) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialOrd for Primer {
    fn partial_cmp(&self, other: &Primer) -> Option<Ordering> {
        Some(self.as_bytes().cmp(other.as_bytes()))
    }
}

impl Hash for Primer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl std::convert::AsRef<[u8]> for Primer {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

struct CustomDist;

impl Distribution<Primer> for CustomDist {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Primer {
        let length = 22 + rng.gen::<u8>() % 4;
        let mut sequence = [Nucleobase::Adenine; 31];
        let mut prev = Nucleobase::Adenine;

        for i in 0..length {
            let next = match (prev, rng.gen::<u8>()) {
                (Nucleobase::Adenine, 0..=128) => Nucleobase::Adenine,
                (Nucleobase::Adenine, 129..=170) => Nucleobase::Cytosine,
                (Nucleobase::Adenine, 171..=240) => Nucleobase::Guanine,
                (Nucleobase::Adenine, 141..=255) => Nucleobase::Thymine,

                (Nucleobase::Cytosine, 0..=80) => Nucleobase::Adenine,
                (Nucleobase::Cytosine, 80..=100) => Nucleobase::Cytosine,
                (Nucleobase::Cytosine, 101..=200) => Nucleobase::Guanine,
                (Nucleobase::Cytosine, 201..=255) => Nucleobase::Thymine,

                (Nucleobase::Guanine, 0..=60) => Nucleobase::Adenine,
                (Nucleobase::Guanine, 61..=80) => Nucleobase::Cytosine,
                (Nucleobase::Guanine, 81..=180) => Nucleobase::Guanine,
                (Nucleobase::Guanine, 181..=255) => Nucleobase::Thymine,

                (Nucleobase::Thymine, 0..=30) => Nucleobase::Adenine,
                (Nucleobase::Thymine, 31..=120) => Nucleobase::Cytosine,
                (Nucleobase::Thymine, 121..=254) => Nucleobase::Guanine,
                (Nucleobase::Thymine, 255..=255) => Nucleobase::Thymine,
            };
            sequence[i as usize] = next;
            prev = next;
        }

        Primer { length, sequence }
    }
}

sortedvec::sortedvec! {
    struct SortedPrimerVec {
        fn derive_key(x: &Primer) -> &Primer { x }
    }
}

sortedvec::sortedvec_slicekey! {
    struct SortedSlicePrimerVec {
        fn derive_key(x: &Primer) -> &[u8] { x.as_bytes() }
    }
}

macro_rules! gen_bench {
    ( $( $i:ident; $sample_size:expr ),* ) => {
        $(mod $i {
            use super::*;

            const SAMPLE_SIZE: usize = $sample_size;
            const SEED: [u8; 16] = [1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16];

            #[bench]
            fn find_primer_naive_sortedvec(b: &mut test::Bencher) {
                // create primer set
                let mut rng = SmallRng::from_seed(SEED);
                let dataset: SortedPrimerVec = rng.sample_iter(&CustomDist).take(SAMPLE_SIZE).collect();
                let test_val = dataset[SAMPLE_SIZE/ 2 - 1];

                b.iter(|| {
                    dataset.find(&&test_val);
                });
            }

            #[bench]
            fn find_primer_list_sortedvec(b: &mut test::Bencher) {
                // check that Primer is of right size
                assert_eq!(std::mem::size_of::<Primer>(), 32);

                // create primer set
                let mut rng = SmallRng::from_seed(SEED);
                let dataset: SortedSlicePrimerVec = rng.sample_iter(&CustomDist).take(SAMPLE_SIZE).collect();
                let test_val = dataset[SAMPLE_SIZE/ 2 - 1];

                b.iter(|| {
                    dataset.find(&test_val);
                });
            }

            #[bench]
            fn find_primer_hashset(b: &mut test::Bencher) {
                // create primer set
                let mut rng = SmallRng::from_seed(SEED);
                let dataset: std::collections::HashSet<_> = rng.sample_iter(&CustomDist).take(SAMPLE_SIZE).collect();
                let test_val = dataset.iter().skip(SAMPLE_SIZE/ 2 - 1).next().unwrap();

                b.iter(|| {
                    dataset.contains(test_val)
                });
            }
        })*
    }
}

gen_bench! {
    dna_e1;10,
    dna_e2;100,
    dna_e3;1_000,
    dna_e4;10_000,
    dna_e5;100_000,
    dna_e6;1_000_000
}
