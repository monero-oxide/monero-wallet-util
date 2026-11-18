//! Coin separation and explicit feature opt-in preserve the Monero defaults.

use polyseed::{Coin, Language, Polyseed, PolyseedError, get_feature};
use zeroize::Zeroizing;

#[test]
fn coin_domains_change_words_and_keys_without_changing_stored_entropy() {
  let seed = Polyseed::from(Language::English, 0, 1_700_000_000, Zeroizing::new([0; 32])).unwrap();
  assert_eq!(seed.key(), seed.key_for_coin(Coin::Monero));
  assert_eq!(seed.to_string(), seed.to_string_for_coin(Coin::Monero));
  for coin in [Coin::Monero, Coin::Aeon, Coin::Wownero] {
    let phrase = seed.to_string_for_coin(coin);
    let recovered =
      Polyseed::from_string_with_options(Language::English, phrase.clone(), coin, 0).unwrap();
    assert_eq!(recovered, seed);
    assert_eq!(
      Polyseed::from_string_auto_with_options(phrase.clone(), coin, 0).unwrap(),
      (seed.clone(), Language::English)
    );
    if coin != Coin::Monero {
      assert_ne!(seed.key_for_coin(coin), seed.key());
      assert_eq!(
        Polyseed::from_string(Language::English, phrase.clone()),
        Err(PolyseedError::InvalidChecksum)
      );
      assert_eq!(Polyseed::from_string_auto(phrase), Err(PolyseedError::InvalidChecksum));
    }
  }
  assert_ne!(seed.key_for_coin(Coin::Aeon), seed.key_for_coin(Coin::Wownero));
}

#[test]
fn user_features_are_explicit_when_creating_and_require_opt_in_when_reading() {
  for features in 1 .. 8 {
    assert_eq!(
      Polyseed::from(Language::English, features, 0, Zeroizing::new([0; 32])),
      Err(PolyseedError::UnsupportedFeatures)
    );
    let seed =
      Polyseed::from_with_features(Language::English, features, 0, Zeroizing::new([0; 32]))
        .unwrap();
    assert_eq!(seed.features(), features);
    assert_eq!(get_feature(seed.features(), features), features);
    assert_eq!(get_feature(seed.features(), 0xff), features);
    assert_eq!(
      Polyseed::from_string(Language::English, seed.to_string()),
      Err(PolyseedError::UnsupportedFeatures)
    );
    assert_eq!(
      Polyseed::load(&seed.store(), Language::English),
      Err(PolyseedError::UnsupportedFeatures)
    );
    assert_eq!(
      Polyseed::from_string_auto(seed.to_string()),
      Err(PolyseedError::UnsupportedFeatures)
    );
    assert_eq!(
      Polyseed::from_string_auto_with_options(seed.to_string(), Coin::Monero, features).unwrap(),
      (seed.clone(), Language::English)
    );
    assert_eq!(
      Polyseed::from_string_with_options(
        Language::English,
        seed.to_string(),
        Coin::Monero,
        features
      )
      .unwrap(),
      seed
    );
    assert_eq!(
      Polyseed::load_with_features(&seed.store(), Language::English, features).unwrap(),
      seed
    );
    assert_eq!(
      Polyseed::from_string_with_options(Language::English, seed.to_string(), Coin::Monero, 0xf8),
      Err(PolyseedError::UnsupportedFeatures)
    );
  }
}

#[test]
fn aeon_feature_key_matches_reference_c_kdf_inputs() {
  // Fixed entropy and salt from tevador/polyseed tests/tests.c, pbkdf2_dummy3.
  // Expected PBKDF2-HMAC-SHA256 output computed independently with Python hashlib.
  let entropy = hex::decode("67b936dfa4da6ae8d3b3cdb3b937f4027b0e3b00000000000000000000000000")
    .unwrap()
    .try_into()
    .unwrap();
  let seed =
    Polyseed::from_with_features(Language::English, 1, 4_305_268_800, Zeroizing::new(entropy))
      .unwrap();
  assert_eq!(
    hex::encode(seed.key_for_coin(Coin::Aeon).as_slice()),
    "c592f2b5998f37f22ac7abc3473b3a9aadb2ff57b67fee8d555ab6123e8741ac"
  );
}
