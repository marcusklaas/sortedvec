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
mod slicekey_bench {
    static LIB: &[&str] = &[
        "1862_Chicago_mayoral_election",
        "1862_City_of_Auckland_West_by-election",
        "1862_City_of_Dunedin_by-election",
        "1862_City_of_Dunedin_by-elections",
        "1862_Coleraine_by-election",
        "1862_Constitution_of_Liechtenstein",
        "1862_Dakota_War",
        "1862_Drayton_and_Toowoomba_colonial_by-election",
        "1862_Dunedin_by-election",
        "1862_Dunedin_by-elections",
        "1862_Eastern_Downs_colonial_by-election",
        "1862_Ellesmere_by-election",
        "1862_English_cricket_season",
        "1862_Grand_National",
        "1862_Greek_head_of_state_referendum",
        "1862_Greek_legislative_election",
        "1862_Hampden_by-election",
        "1862_Heathcote_by-election",
        "1862_International_Exhibition",
        "1862_Liverpool_Town_Council_election",
        "1862_Montgomeryshire_by-election",
        "1862_New_Jersey_gubernatorial_election",
        "1862_New_Plymouth_by-election",
        "1862_New_York_state_election",
        "1862_Norwegian_parliamentary_election",
        "1862_Open_Championship",
        "1862_Oregon_gubernatorial_election",
        "1862_South_Australian_colonial_election",
        "1862_Town_of_New_Plymouth_by-election",
        "1862_United_States_House_of_Representatives_election_in_Delaware",
        "1862_United_States_House_of_Representatives_election_in_Kansas",
        "1862_United_States_House_of_Representatives_election_in_Oregon",
        "1862_United_States_House_of_Representatives_elections_in_Illinois",
        "1862_United_States_House_of_Representatives_elections_in_Indiana",
        "1862_United_States_House_of_Representatives_elections_in_Iowa",
        "1862_United_States_House_of_Representatives_elections_in_Maine",
        "1862_United_States_House_of_Representatives_elections_in_Massachusetts",
        "1862_United_States_House_of_Representatives_elections_in_Michigan",
        "1862_United_States_House_of_Representatives_elections_in_Minnesota",
        "1862_United_States_House_of_Representatives_elections_in_Missouri",
        "1862_United_States_House_of_Representatives_elections_in_New_Jersey",
        "1862_United_States_House_of_Representatives_elections_in_New_York",
        "1862_United_States_House_of_Representatives_elections_in_Ohio",
        "1862_United_States_House_of_Representatives_elections_in_Pennsylvania",
        "1862_United_States_House_of_Representatives_elections_in_Wisconsin",
        "1862_United_States_Senate_election_in_Indiana",
        "1862_United_States_Senate_election_in_Missouri",
        "1862_United_States_Senate_election_in_Rhode_Island",
        "1862_United_States_Senate_election_in_Vermont",
        "1862_United_States_Senate_special_election_in_Michigan",
        "1862_United_States_Senate_special_election_in_Oregon",
        "1862_United_States_Senate_special_election_in_Rhode_Island",
        "1862_United_States_elections",
        "1862_Virginia's_1st_congressional_district_special_election",
        "1862_Wardsend_Cemetery_riot",
        "1862_Warwick_colonial_by-election",
        "1862_West_Moreton_colonial_by-election",
        "1862_and_1863_United_States_House_of_Representatives_elections",
        "1862_and_1863_United_States_Senate_elections",
        "1862_congressional_elections",
        "1862_in_Australia",
        "1862_in_Australian_literature",
        "1862_in_Belgium",
        "1862_in_Brazil",
        "1862_in_Canada",
        "1862_in_Chile",
        "1862_in_China",
        "1862_in_Denmark",
        "1862_in_France",
        "1862_in_Germany",
        "1862_in_India",
        "1862_in_Ireland",
        "1862_in_Mexico",
        "1862_in_New_Zealand",
        "1862_in_Norway",
        "1862_in_Portugal",
        "1862_in_Russia",
        "1862_in_Scotland",
        "1862_in_Siam",
        "1862_in_South_Africa",
        "1862_in_Sweden",
        "1862_in_Wales",
        "1862_in_archaeology",
        "1862_in_architecture",
        "1862_in_art",
        "1862_in_baseball",
        "1862_in_birding_and_ornithology",
        "1862_in_film",
        "1862_in_literature",
        "1862_in_music",
        "1862_in_paleontology",
        "1862_in_poetry",
        "1862_in_rail_transport",
        "1862_in_science",
        "1862_in_sociology",
        "1862_in_sports",
        "1862_in_the_American_Old_West",
        "1862_in_the_UK",
        "1862_in_the_US",
        "1862_in_the_USA",
        "1862_in_the_United_Kingdom",
        "1862_in_the_United_States",
        "1862_in_the_United_States_of_America",
        "1862–1864_Atlantic_hurricane_seasons",
        "1862–1910_Argentine_presidential_elections",
        "1863",
        "1863-1875_cholera_pandemic",
        "1863-64_Australian_cricket_season",
        "1863-64_New_Zealand_cricket_season",
        "1863-75_cholera_pandemic",
        "18631_Maurogherardini",
        "18632_Danielsson",
        "18634_Champigneulles",
        "18635_Frouard",
        "18636_Villedepompey",
        "18637_Liverdun",
        "18638_Nouet",
        "18639_Aoyunzhiyuanzhe",
        "1863_(number)",
        "1863_(year)",
        "1863_AD",
        "1863_AHS",
        "1863_Akaroa_by-election",
        "1863_Antinous",
        "1863_Atlantic_hurricane_season",
        "1863_BC",
        "1863_Belgian_general_election",
        "1863_CE",
        "1863_California_gubernatorial_election",
        "1863_Chicago_mayoral_election",
        "1863_Colony_of_Vancouver_Island_election",
        "1863_Confederate_Congressional_election",
        "1863_Confederate_States_House_of_Representatives_elections",
        "1863_Costa_Rican_general_election",
        "1863_Delaware's_at-large_congressional_district_special_election",
        "1863_Dunedin_and_Suburbs_North_by-election",
        "1863_Dunedin_and_Suburbs_South_by-election",
        "1863_East_Moreton_colonial_by-election",
        "1863_English_cricket_season",
        "1863_French_legislative_election",
        "1863_Grand_National",
        "1863_Hampden_by-election",
        "1863_Hawke's_Bay_earthquake",
        "1863_Heathcote_by-election",
        "1863_Insurrection",
        "1863_Jujuy_earthquake",
        "1863_Kaiapoi_by-election",
        "1863_Liberian_general_election",
        "1863_Liverpool_Town_Council_election",
        "1863_Louisiana_gubernatorial_election_(Confederate)",
        "1863_Mexican_emperor_referendum",
        "1863_Minnesota_gubernatorial_election",
        "1863_New_Plymouth_by-election",
        "1863_New_York_City_draft_riots",
        "1863_New_York_state_election",
        "1863_Open_Championship",
        "1863_Orange_Free_State_presidential_election",
        "1863_Pennsylvania_gubernatorial_election",
        "1863_Polish_revolution",
        "1863_Port_Curtis_colonial_by-election",
        "1863_Queensland_colonial_election",
        "1863_State_of_the_Union",
        "1863_State_of_the_Union_Address",
        "1863_Swiss_federal_election",
        "1863_Texas_gubernatorial_election",
        "1863_Town_of_New_Plymouth_by-election",
        "1863_United_States_House_of_Representatives_election_in_California",
        "1863_United_States_House_of_Representatives_elections_in_California",
        "1863_United_States_House_of_Representatives_elections_in_Connecticut",
        "1863_United_States_House_of_Representatives_elections_in_Kentucky",
        "1863_United_States_House_of_Representatives_elections_in_Maryland",
        "1863_United_States_House_of_Representatives_elections_in_New_Hampshire",
        "1863_United_States_House_of_Representatives_elections_in_Rhode_Island",
        "1863_United_States_House_of_Representatives_elections_in_Vermont",
        "1863_United_States_House_of_Representatives_elections_in_West_Virginia",
        "1863_United_States_Senate_election_in_Connecticut",
        "1863_United_States_Senate_election_in_Delaware",
        "1863_United_States_Senate_election_in_Maine",
        "1863_United_States_Senate_election_in_Massachusetts",
        "1863_United_States_Senate_election_in_Michigan",
        "1863_United_States_Senate_election_in_Minnesota",
        "1863_United_States_Senate_election_in_New_York",
        "1863_United_States_Senate_election_in_Ohio",
        "1863_United_States_Senate_election_in_Pennsylvania",
        "1863_United_States_Senate_election_in_Virginia",
        "1863_United_States_Senate_election_in_Wisconsin",
        "1863_United_States_Senate_elections_in_West_Virginia",
        "1863_United_States_Senate_special_election_in_Illinois",
        "1863_United_States_Senate_special_election_in_Indiana",
        "1863_United_States_Senate_special_election_in_Missouri",
        "1863_United_States_Senate_special_election_in_New_Jersey",
    ];

    sortedvec::sortedvec! {
        /// Sorted vector type that provides quick access to `T`s through `K`s.
        #[derive(Debug, Clone)]
        pub struct SortedVec {
            fn sort_key(t: &String) -> &str { &t[..] }
        }
    }

    #[bench]
    fn find_wiki_article_regular(b: &mut test::Bencher) {
        let sortedvec: SortedVec = LIB.into_iter().map(|&x| x.to_owned()).collect();

        b.iter(|| sortedvec.find(&"1862_United_States_House_of_Representatives_elections_in_New_Jersey"));
    }

    use sortedvec::list_experiment::*;

    sortedvec::sortedvec_slicekey! {
        /// Sorted vector type that provides quick access to `T`s through `K`s.
        #[derive(Debug, Clone)]
        pub struct SortedVecSliceKey {
            fn sort_key_slice(t: &String) -> &[u8] { t.as_bytes() }
        }
    }

    #[bench]
    fn find_wiki_article_slicekey(b: &mut test::Bencher) {
        let sortedvec: SortedVecSliceKey = LIB.into_iter().map(|&x| x.to_owned()).collect();

        b.iter(|| sortedvec.find("1862_United_States_House_of_Representatives_elections_in_New_Jersey"));
    }

    const TEST_STRING_A: &str = "1862_United_States_House_of_Representatives_elections_in_New_Jersey";
    const TEST_STRING_B: &str = "1862_United_States_House_of_Representatives_elections_in_New_York";

    #[bench]
    fn simple_str_cmp(bench: &mut test::Bencher) {
        bench.iter(|| TEST_STRING_A.cmp(TEST_STRING_B));
    }

    #[bench]
    fn prefix_len_str_cmp(bench: &mut test::Bencher) {
        bench.iter(|| simd_common_prefix_len(TEST_STRING_A.as_bytes(), TEST_STRING_B.as_bytes()));
    }
}

#[cfg(test)]
mod dna_primers {
    use rand::prelude::*;
    use std::cmp::Ordering;
    
    #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
    enum Nucleobase {
        Adenine,
        Cytosine,
        Guanine,
        Thymine,
    }

    impl Nucleobase {
        fn from_u8(x: u8) -> Self {
            match x {
                0...1 => Nucleobase::Adenine,
                2...3 => Nucleobase::Cytosine,
                4...5 => Nucleobase::Guanine,
                _     => Nucleobase::Thymine,
            }
        }
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

    sortedvec::sortedvec_slicekey! {
        struct SortedByteSlicePrimerVec {
            fn key_deriv_three(x: &Primer) -> &[u8] { x.as_bytes() }
        }
    }

    #[bench]
    fn find_primer_list_as_bytes(b: &mut test::Bencher) {
        let seed = [1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16];
        let mut rng = SmallRng::from_seed(seed);
        let dataset: SortedByteSlicePrimerVec = rng.sample_iter(&CustomDist).take(SAMPLE_SIZE).collect();
        let test_val = dataset[SAMPLE_SIZE/ 2 - 1];

        assert!(std::mem::size_of::<Nucleobase>() == 1);

        b.iter(|| {
            dataset.find(test_val.as_bytes());
        });
    }
}

