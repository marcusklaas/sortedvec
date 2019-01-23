use crate::sortedvec;

/// Example key
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
pub struct K;

/// Example value
#[derive(Debug, Clone)]
pub struct T {
    key: K,
}

sortedvec! {
    /// Sorted vector type that provides quick access to `T`s through `K`s.
    #[derive(Debug, Clone)]
    pub struct ExampleSortedVec {
        fn key(t: &T) -> K { t.key }
    }
}
