//! Polyseed encryption interoperability and password normalization.

use polyseed::{Language, Polyseed};
use zeroize::Zeroizing;

fn reference_seed() -> Polyseed {
  Polyseed::from_string(
    Language::English,
    Zeroizing::new(
      "unaware yard donate shallow slot sing oil oxygen \
     loyal bench near hill surround forum execute lamp"
        .into(),
    ),
  )
  .unwrap()
}

#[test]
fn matches_dart_encryption_vector_and_recovers_from_encrypted_words() {
  let original = reference_seed();
  let mut encrypted = original.clone();
  encrypted.crypt("CakeWallet");
  assert!(encrypted.is_encrypted());
  assert_eq!(hex::encode(&encrypted.entropy()[.. 19]), "444d5b411f8e34e5ff2c1b37dc2ef352764b3c");
  assert_eq!(
    *encrypted.to_string(),
    "arm dutch crystal reduce elephant mix squeeze garlic \
     slam brand tent seed rubber fame summer sample"
  );
  let mut recovered = Polyseed::from_string(Language::English, encrypted.to_string()).unwrap();
  recovered.crypt("CakeWallet");
  assert!(!recovered.is_encrypted());
  assert_eq!(recovered, original);
  assert_eq!(recovered.key(), original.key());
}

#[test]
fn normalizes_passwords_and_does_not_authenticate_them() {
  let original = reference_seed();
  let mut encrypted = original.clone();
  encrypted.crypt("café");
  let mut recovered = encrypted.clone();
  recovered.crypt("cafe\u{301}");
  assert_eq!(recovered, original);
  encrypted.crypt("incorrect");
  assert!(!encrypted.is_encrypted());
  assert_ne!(encrypted.entropy(), original.entropy());
}
