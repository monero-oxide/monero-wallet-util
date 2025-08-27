use zeroize::Zeroizing;
use rand_core::OsRng;

use curve25519_dalek::{constants::ED25519_BASEPOINT_TABLE, Scalar};

use monero_wallet::{address::Network, ViewPair};

use crate::OutProof;

#[test]
fn out_proof_serialization() {
  let spend_key = Zeroizing::new(Scalar::random(&mut OsRng));
  let view_key = Zeroizing::new(Scalar::random(&mut OsRng));
  let view_pair = ViewPair::new(ED25519_BASEPOINT_TABLE * &*spend_key, view_key).unwrap();

  let ephemeral_key = Zeroizing::new(Scalar::random(&mut OsRng));

  let proof =
    OutProof::prove(&mut OsRng, view_pair.legacy_address(Network::Mainnet), &ephemeral_key, &[]);

  let mut proofs = vec![];
  for _ in 0 .. 5 {
    assert_eq!(&OutProof::read(&OutProof::serialize(&proofs)).unwrap(), &proofs);
    proofs.push(proof);
  }
}
