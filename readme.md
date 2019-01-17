# sortedvec

A pure rust library that exposes a single data type, `SortedVec`. Its raison d'être is to
provide a lookup table that has quicker lookups than regular `Vec`s, `O(log(n))` vs `O(n)`,
and is simpler and more memory efficient than hashmaps. It is ideal for (very) small
lookup tables where additions and deletions are infrequent.

## Example

```rust
use sortedvec::SortedVec;

let unsorted = vec![3, 5, 0, 10, 7, 1];
let sorted = SortedVec::from_vec(unsorted.clone(), |&x| x);

// linear search (slow!)
let unsorted_contains_six: Option<_> = unsorted.iter().find(|&x| *x == 6);
assert!(sorted_contains_six.is_none());

// binary search (fast!)
let sorted_contains_six: Option<_> = sorted.find(&6);
assert!(sorted_contains_six.is_none());
```