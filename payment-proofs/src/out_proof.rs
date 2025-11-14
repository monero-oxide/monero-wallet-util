use core::{ops::Deref, fmt};
use std_shims::{string::String, vec::Vec, vec};

use zeroize::{Zeroize, Zeroizing};
use rand_core::{RngCore, CryptoRng};

use sha3::{Digest, Keccak256};
use curve25519_dalek::{
  constants::{ED25519_BASEPOINT_POINT, ED25519_BASEPOINT_TABLE},
  scalar::Scalar,
  edwards::EdwardsPoint,
};

use monero_base58 as base58;
use monero_wallet::{
  transaction::Transaction,
  address::Address,
  extra::{PaymentId, Extra},
};

use crate::shared_key_derivations::SharedKeyDerivations;

/// A Monero OutProof.
///
/// This is specifically a v2 OutProof.
/// [v1 OutProofs were insecure](https://github.com/monero-project/research-lab/issues/60) and are
/// unsupported.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Zeroize)]
pub struct OutProof {
  ecdh: EdwardsPoint,
  c: Scalar,
  s: Scalar,
}

impl OutProof {
  fn challenge<const ADDRESS_BYTES: u128>(
    address: Address<{ ADDRESS_BYTES }>,
    nonce_commitment_generator: EdwardsPoint,
    nonce_commitment_view_key: EdwardsPoint,
    ephemeral_key_commitment: EdwardsPoint,
    ecdh: EdwardsPoint,
    message: &[u8],
  ) -> Scalar {
    let mut keccak = Keccak256::new();
    // msg
    keccak.update(monero_wallet::primitives::keccak256(message));
    // D
    keccak.update(ecdh.compress().to_bytes());
    // X
    keccak.update(nonce_commitment_generator.compress().to_bytes());
    // Y
    keccak.update(nonce_commitment_view_key.compress().to_bytes());
    // sep
    if !address.is_guaranteed() {
      keccak.update(monero_wallet::primitives::keccak256(b"TXPROOF_V2"));
    } else {
      keccak.update(monero_wallet::primitives::keccak256(b"TXPROOF_V2_GUARANTEED"));
    }
    // R
    keccak.update(ephemeral_key_commitment.compress().to_bytes());
    // A
    keccak.update(address.view().compress().to_bytes());
    // B
    if address.is_subaddress() {
      keccak.update(address.spend().compress().to_bytes());
    } else {
      keccak.update([0; 32]);
    }
    Scalar::from_bytes_mod_order(keccak.finalize().into())
  }

  /// Prove an OutProof.
  ///
  /// This will create an OutProof v2.
  ///
  /// This does support proving OutProofs to guaranteed addresses. Such OutProofs have a distinct
  /// DST and will only be verifiable by other wallet software which support proofs to guaranteed
  /// addresses.
  pub fn prove<const ADDRESS_BYTES: u128>(
    rng: &mut (impl RngCore + CryptoRng),
    address: Address<{ ADDRESS_BYTES }>,
    ephemeral_key: &Zeroizing<Scalar>,
    message: &[u8],
  ) -> Self {
    let nonce = Zeroizing::new(Scalar::random(rng));
    let commitment_generator =
      if address.is_subaddress() { address.spend() } else { ED25519_BASEPOINT_POINT };
    let commit = |value: &Zeroizing<Scalar>| {
      (commitment_generator * value.deref(), address.view() * value.deref())
    };
    let (nonce_commitment_generator, nonce_commitment_view_key) = commit(&nonce);
    let (ephemeral_key_commitment, ecdh) = commit(ephemeral_key);
    let c = Self::challenge(
      address,
      nonce_commitment_generator,
      nonce_commitment_view_key,
      ephemeral_key_commitment,
      ecdh,
      message,
    );
    let s = nonce.deref() - (c * ephemeral_key.deref());
    OutProof { ecdh, c, s }
  }

  /// Verify an OutProof.
  ///
  /// This returns the amount transfered if the proof and associated transaction data is valid.
  /// This function does not perform any checks on the transaction's `additional_timelock` field.
  // TODO: Make Timelocked generic and public
  pub fn verify<const ADDRESS_BYTES: u128>(
    self,
    tx: &Transaction,
    output_index: usize,
    address: Address<{ ADDRESS_BYTES }>,
    message: &[u8],
  ) -> Option<u64> {
    let commitment_generator =
      if address.is_subaddress() { address.spend() } else { ED25519_BASEPOINT_POINT };

    let OutProof { ecdh, c, s } = self;
    let s_commitment_generator = commitment_generator * s;
    let s_commitment_view_key = address.view() * s;

    let extra = Extra::read(&mut tx.prefix().extra.as_slice()).ok()?;
    let (keys, additional_keys) = extra.keys()?;
    let mut keys = keys
      .into_iter()
      .map(Some)
      .chain(core::iter::once(additional_keys.and_then(|keys| keys.get(output_index).copied())));

    while let Some(Some(key)) = keys.next() {
      if c ==
        Self::challenge(
          address,
          s_commitment_generator + (c * key),
          s_commitment_view_key + (c * ecdh),
          key,
          ecdh,
          message,
        )
      {
        let output = tx.prefix().outputs.get(output_index)?;

        let shared_key_derivations = SharedKeyDerivations::output_derivations(
          address.is_guaranteed().then(|| SharedKeyDerivations::uniqueness(&tx.prefix().inputs)),
          Zeroizing::new(ecdh),
          output_index,
        );

        // Check the view tag
        if let Some(actual_view_tag) = output.view_tag {
          if actual_view_tag != shared_key_derivations.view_tag {
            None?;
          }
        }

        // Check the output's key
        if output.key !=
          (address.spend() + (&shared_key_derivations.shared_key * ED25519_BASEPOINT_TABLE))
            .compress()
        {
          None?;
        }

        // Check the payment ID, if one was expected
        if let Some(payment_id) = address.payment_id() {
          if PaymentId::Encrypted(payment_id) !=
            (extra.payment_id()? ^ SharedKeyDerivations::payment_id_xor(Zeroizing::new(ecdh)))
          {
            None?;
          }
        }

        // Fetch the amount to return
        return Some(if let Some(amount) = output.amount {
          // v1/miner transaction
          amount
        } else {
          // v2 non-miner transaction without proofs, which shouldn't exist
          let Transaction::V2 { proofs: Some(ref proofs), .. } = &tx else { None? };

          // Decrypt the amount
          let commitment =
            shared_key_derivations.decrypt(proofs.base.encrypted_amounts.get(output_index)?);

          // Rebuild the commitment to verify it
          if Some(&commitment.calculate()) != proofs.base.commitments.get(output_index) {
            None?;
          }

          commitment.amount
        });
      }
    }

    None
  }

  /// Serialize a series of OutProofs to a single string.
  pub fn serialize(proofs: &[Self]) -> String {
    let mut res = String::with_capacity(10 + (proofs.len() * 96));
    res.push_str("OutProofV2");
    for proof in proofs {
      let OutProof { ecdh, c, s } = proof;
      let mut signature = c.to_bytes().to_vec();
      signature.extend(&s.to_bytes());
      res.push_str(&base58::encode(&ecdh.compress().to_bytes()));
      res.push_str(&base58::encode(&signature));
    }
    res
  }

  /// Read a Vec of OutProofs from a str.
  ///
  /// Returns None if the encoding was invalid.
  pub fn read(proofs: &str) -> Option<Vec<Self>> {
    let ecdh_len = base58::encode(&[0; 32]).len();
    let signature_len = base58::encode(&[0; 64]).len();

    let mut res = vec![];
    if let Some(mut proofs) = proofs.strip_prefix("OutProofV2") {
      while !proofs.is_empty() {
        if proofs.len() < (ecdh_len + signature_len) {
          None?;
        }
        let ecdh = base58::decode(proofs.get(.. ecdh_len)?)?;
        #[allow(clippy::string_slice)] // Safe via immediately prior `get`
        {
          proofs = &proofs[ecdh_len ..];
        }

        let signature = base58::decode(proofs.get(.. signature_len)?)?;
        #[allow(clippy::string_slice)] // Safe via immediately prior `get`
        {
          proofs = &proofs[signature_len ..];
        }

        res.push(OutProof {
          ecdh: monero_wallet::io::read_point(&mut ecdh.as_slice()).ok()?,
          c: monero_wallet::io::read_scalar(&mut &signature[.. 32]).ok()?,
          s: monero_wallet::io::read_scalar(&mut &signature[32 ..]).ok()?,
        });
      }
    }
    Some(res)
  }
}

impl fmt::Display for OutProof {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let OutProof { ecdh, c, s } = self;
    let mut signature = c.to_bytes().to_vec();
    signature.extend(&s.to_bytes());
    write!(
      f,
      "OutProofV2{}{}",
      base58::encode(&ecdh.compress().to_bytes()),
      base58::encode(&signature)
    )
  }
}
