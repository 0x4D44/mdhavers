# Muckle Numbers - an exact-integer toolkit

mdhavers integers are exact 64-bit integers on every backend. Nothing rounds
past 2^53. A result that overflows raises a Scots error, and `hae_a_bash`
can catch it. This programme pushes that model as far as it goes:

- factorials up to 20!, and the overflow at 21!
- every Fibonacci number that fits (up to F(92)), with Cassini's identity
  checked exactly until its products overflow at n = 48
- the highest power of each base from 2 to 10 that fits in 64 bits, one fresh
  closure per base
- Collatz record-breakers under 1000, tracked in a dict that starts empty
  (and so falsy)
- a Sieve of Eratosthenes, twin primes and the widest prime gap under 1000
- a Lucas-Lehmer test with an overflow-free `mulmod`, which proves
  2^61 - 1 prime
- a summary dict compared with `==` against the textbook values

## Running

```bash
mdhavers run examples/showcases/muckle_numbers/muckle_numbers.braw

# JavaScript (BigInt integers)
mdhavers compile examples/showcases/muckle_numbers/muckle_numbers.braw -o muckle.js
node muckle.js

# Native (LLVM)
mdhavers build examples/showcases/muckle_numbers/muckle_numbers.braw -o muckle
./muckle
```

The interpreter, JavaScript and native output match byte for byte. The WASM
backend cannot compile it yet, because it has no builtins, `kin` or
`hae_a_bash`. The debug interpreter runs it in about 0.25 s.

The golden test `tests/golden/integration/showcase_muckle_numbers.braw` is a
copy with a header, so its overflow messages carry that file's line numbers.
