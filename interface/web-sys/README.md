# Monero web-sys RPC

A [connection](https://docs.rs/monero-interface) to a
[Monero daemon](https://docs.rs/monero-daemon-rpc) via
[`web-sys`](https://docs.rs/web-sys), built around
[`monero-oxide`](https://docs.rs/monero-oxide).

This is intended for use within a single-threaded Javascript runtime and is
unsafe to use in a multi-threaded context.
