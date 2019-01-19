#!/bin/sh

cargo bench | sort | grep "+/- " | sed 's/test //' | tr -s " " \
    | sed 's/ ns.*//' | sed 's/find_.*://' \
    | sed -E '$!N;/^(\S+ )(.*)\n\1/!P;s//\n\1\2 | /;D' \
    | sed 's/_bench::s/ |/' | sed 's/^/| /' | sed 's/$/ |/' \
    | sed 's/:: / | /' | sed  's/|0\+/| /' \
    | (echo "|---|---:|---:|---:|---:|" && cat) \
    | (echo "| key type | size | `HashMap` | `SortedVec` | `Vec` |" && cat) > bench_results.txt
