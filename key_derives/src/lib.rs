#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

#[cfg(test)]
mod tests;

#[cfg(feature = "monero-wallet")]
use monero_wallet::ViewPair;

use sha3::{Digest, Keccak256};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};
use curve25519_dalek::{EdwardsPoint, Scalar, constants::ED25519_BASEPOINT_TABLE};

#[derive(Zeroize, ZeroizeOnDrop)]
/// Legacy CryptoNote wallet structure
pub struct LegacyWallet {
  priv_spend: Zeroizing<Scalar>,
  pub_spend: EdwardsPoint,

  priv_view: Zeroizing<Scalar>,
  pub_view: EdwardsPoint,
}

impl LegacyWallet {
  /// Derives wallet keys from a seed, using the legacy CryptoNote algorithm
  ///
  /// Returns a LegacyWallet containing the spend key-pair and view key-pair
  pub fn new(seed: &[u8; 32]) -> LegacyWallet {
    let mut priv_spend = Scalar::from_bytes_mod_order(*seed);
    let pub_spend = &priv_spend * ED25519_BASEPOINT_TABLE;

    let mut priv_view = Scalar::from_bytes_mod_order(Keccak256::digest(*seed).into());
    let pub_view = &priv_view * ED25519_BASEPOINT_TABLE;

    let wallet = LegacyWallet {
      priv_spend: priv_spend.into(),
      pub_spend,
      priv_view: priv_view.into(),
      pub_view,
    };

    // Zeroize all the private variables
    priv_spend.zeroize();
    priv_view.zeroize();

    wallet
  }

  /// Returns the spend key-pair of the wallet.
  pub fn spend(&self) -> (Zeroizing<Scalar>, EdwardsPoint) {
    (self.priv_spend.clone(), self.pub_spend)
  }

  /// Returns the view key-pair of the wallet.
  pub fn view(&self) -> (Zeroizing<Scalar>, EdwardsPoint) {
    (self.priv_view.clone(), self.pub_view)
  }

  /// Returns a ViewPair from the wallet's private view key and public spend key, for use with `monero-wallet`.
  #[cfg(feature = "monero-wallet")]
  pub fn view_pair(&self) -> Result<ViewPair, monero_wallet::ViewPairError> {
    use monero_wallet::ed25519::Point;

    ViewPair::new(
      Point::from(self.pub_spend),
      Zeroizing::new(monero_wallet::ed25519::Scalar::from(*self.priv_view)),
    )
  }
}
