# wildflower

[![workflow status](https://github.com/cassaundra/wildflower/actions/workflows/main.yml/badge.svg)](https://github.com/cassaundra/wildflower/actions)
[![crates.io](https://img.shields.io/crates/v/wildflower)](https://crates.io/crates/wildflower)
[![docs.rs](https://img.shields.io/docsrs/wildflower)](https://docs.rs/wildflower/latest/wildflower/)

![(kitten trying to eat some some flowers that look suspicously like asterisks)](wildflower_cat.jpg)

wildflower is a Rust library that performs [wildcard matching](https://en.wikipedia.org/wiki/Matching_wildcards) against strings.
It's fast, ergonomic, zero-copy, and works on `no_std`.

## Usage

The wildcard matching grammar contains the following special characters:

- `?` matches a single character.
- `*` matches zero or more characters.
- `\` escapes these special characters.

A pattern is constructed from a UTF-8-encoded string which may contain these special characters.
When a pattern is created, the given source string is parsed and compiled into an optimized internal form.
Since no internal state is maintained between matches, it is recommended that you reuse patterns for best results.

## Alternatives

[wildmatch](https://crates.io/crates/wildmatch) is the closest alternative at the time of writing.
Unfortunately, it explicitly does not support escaped special characters, and is not particularly optimized (see [benchmarks](#benchmarking)).

Several other crates exist for pattern matching, namely [regex](https://crates.io/crates/regex) (for regular expressions) and [glob](https://crates.io/crates/glob) (for Unix shell patterns).

## Benchmarking

Using a benchmark similar to the one found in wildmatch ([source](https://github.com/becheran/wildmatch/blob/master/benches/patterns.rs)), I obtained the following results on my machine:

| Benchmark         | wildflower | wildmatch |      regex |     glob |
|-------------------|-----------:|----------:|-----------:|---------:|
| compiling/text    |     354 ns |    386 ns | 129,180 ns | 1,875 ns |
| compiling/complex |     219 ns |     44 ns |  79,536 ns |   140 ns |
| matching/text     |       7 ns |    531 ns |     433 ns |   797 ns |
| matching/complex  |     132 ns |    558 ns |     407 ns | 2,193 ns |

In this benchmark run, wildflower is shown to be 76x and 4x as fast as wildmatch in the simple and complex case of matching respectfully.
It could certainly stand to see performance improvements in compiling, but even in the worst case of a single-use compilation, it still outperforms wildmatch.

## Credits

Credit to [Armin Becher](https://github.com/becheran) for the benchmarking code and table format from wildmatch, and to [Ilona Ily??s](https://pixabay.com/users/ilonaburschl-3558510/) of Pixabay for the original of the cat image featured above.
