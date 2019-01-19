# sortedvec

A pure rust library that exposes a single data type, `SortedVec`. Its raison d'être is to
provide a lookup table that has quicker lookups than regular `Vec`s, `O(log(n))` vs `O(n)`,
and is simpler and more memory efficient than hashmaps. It is ideal for (very) small
lookup tables where additions and deletions are infrequent.

## Example

```rust
use sortedvec::SortedVec;

let unsorted = vec![3, 5, 0, 10, 7, 1];
let sorted = SortedVec::from_vec(unsorted.clone(), |x| x);

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
| int | 2 | 28 | 1 | 1 |
| int | 6 | 29 | 3 | 2 |
| int | 10 | 28 | 4 | 3 |
| int | 50 | 28 | 6 | 13 |
| int | 100 | 33 | 7 | 25 |
| int | 500 | 28 | 9 | 130 |
| int |1000 | 28 | 10 | 245 |
| string | 2 | 32 | 13 | 5 |
| string | 6 | 31 | 24 | 13 |
| string | 10 | 32 | 30 | 23 |
| string | 50 | 33 | 44 | 123 |
| string | 100 | 31 | 51 | 231 |
| string | 500 | 32 | 67 | 1,149 |
| string |1000 | 32 | 73 | 2,328 |
