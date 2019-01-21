# sortedvec

A pure rust library that exposes a single macro, [`def_sorted_vec`]. It generates a lookup
table on `Ord` keys that has quicker lookups than regular `Vec`s, `O(log(n))` vs `O(n)`,
and is simpler and more memory efficient than hashmaps. It is ideal for (very) small
lookup tables where insertions and deletions are infrequent.

**Note**: `sortedvec` is still highly experimental and likely to change significantly.

## Example

```rust
use sortedvec::def_sorted_vec;

def_sorted_vec! { struct SortedVec: u32 => u32, |x| x }

let unsorted = vec![3, 5, 0, 10, 7, 1];
let sorted = SortedVec::from(unsorted.clone());

// linear search (slow!)
let unsorted_contains_six: Option<_> = unsorted.iter().find(|&x| *x == 6);
assert!(unsorted_contains_six.is_none());

// binary search (fast!)
let sorted_contains_six: Option<_> = sorted.find(&6);
assert!(sorted_contains_six.is_none());
```

## Benchmarks

The table below displays how lookups scale on the standard library's `HashMap`,
`SortedVec` and `Vec` for string and integer keys.

| key type | size | `HashMap` | `SortedVec` | `Vec` |
|---|---:|---:|---:|---:|
| int | 2 | 17 | 2 | 2 |
| int | 6 | 17 | 3 | 2 |
| int | 10 | 18 | 4 | 3 |
| int | 50 | 19 | 5 | 15 |
| int | 100 | 23 | 6 | 28 |
| int | 500 | 18 | 8 | 127 |
| int |1000 | 17 | 8 | 231 |
| string | 2 | 25 | 10 | 5 |
| string | 6 | 25 | 20 | 12 |
| string | 10 | 27 | 25 | 21 |
| string | 50 | 30 | 36 | 113 |
| string | 100 | 27 | 42 | 232 |
| string | 500 | 26 | 53 | 1,207 |
| string |1000 | 26 | 59 | 2,324 |
