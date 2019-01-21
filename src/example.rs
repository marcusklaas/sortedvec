use crate::def_sorted_vec;

/// Example key
#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct K;

/// Example value
pub struct T {
    key: K,
}

fn key(t: &T) -> &K { &t.key }

def_sorted_vec! {
    /// Sorted vector type that provides quick access to `T`s through `K`s.
    pub struct ExampleSortedVec: T => K, key
}
