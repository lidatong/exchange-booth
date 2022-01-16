# Exchange Booth

## Summary

i setup Rust integration tests (`cargo test-bpf -- --show-output`) to basically automate the whole process of standing up a test validator, building & deploying, running a client against the program, etc.

everything happens all at once in Rust (using `cargo`) and makes it much easier to iterate quickly -- including being able to see your `program logs` inside the terminal with each test run!

if you're interested, here's the link to the test: https://github.com/lidatong/exchange-booth/blob/master/exchange-booth/program/tests/integration.rs (forgive the blatant copy pasting). you can also clone the repo and run `cargo test-bpf -- --show-output` to try it.

## Other notes

- my actual exchange booth impl is only happy-path tested -- it's definitely missing checks, buggy, etc.
- setting up a `cargo workspace` allowed me deploy both `echo (oracle)` and `exchange booth` to the same `TestValidatorGenesis`.
- relevant Rust crates: `solana-test-validator`, `solana-logger`, `solana-client`, `solana-sdk`. _note this is different from `solana-program-test` / `BanksClient`_.

