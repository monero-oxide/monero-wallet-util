# monero-wallet-util

Utilities around
[`monero-wallet`](
  https://monero-oxide.github.io/monero-oxide/monero_wallet/index.html
) to facilitate a better experience for developers.

These libraries are less often maintained, with less stability guarantees, than
`monero-wallet` [as hosted in the `monero-oxide` repository](
  https://github.com/monero-oxide/monero-oxide/tree/main/monero-oxide/wallet
). It also may pull in more dependencies in order to provide a more
'out of the box' experience to developers in the Monero ecosystem.

### Current Features

- [Monero Seed](seed)
- [Polyseed](polyseed)

### In Development

- [Monero Payment Proofs](https://github.com/monero-oxide/monero-wallet-util/pull/1)

### Wishlist

- An [`cuprate-epee-encoding`-using RPC](
    https://github.com/monero-oxide/monero-oxide/issues/41
  )
- An [`arti`](https://docs.rs/arti)-premised
  [RPC](https://monero-oxide.github.io/monero-oxide/monero_rpc/index.html)
- [An RPC which uses a _local_ store for the decoys](
    https://github.com/monero-oxide/monero-oxide/issues/34#issuecomment-3217083574
  )
