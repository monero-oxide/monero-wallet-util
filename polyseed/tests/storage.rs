//! Binary storage compatibility and rejection of corrupted encodings.

use polyseed::{Language, Polyseed, PolyseedError};
use zeroize::Zeroizing;

const REFERENCE: &str = "504f4c59534545441600fed04c5330c96679e6121524e6bb6a5b93c733ff6377";

#[test]
fn loads_and_stores_the_dart_reference_vector() {
  let bytes = hex::decode(REFERENCE).unwrap().try_into().unwrap();
  let seed = Polyseed::load(&bytes, Language::English).unwrap();
  assert_eq!(
    *seed.to_string(),
    "unaware yard donate shallow slot sing oil oxygen \
     loyal bench near hill surround forum execute lamp"
  );
  assert_eq!(seed.birthday(), 1_693_622_412);
  assert_eq!(hex::encode(seed.store().as_slice()), REFERENCE);
  let mut encrypted = seed.clone();
  encrypted.crypt("CakeWallet");
  let mut loaded = Polyseed::load(&encrypted.store(), Language::English).unwrap();
  assert_eq!(loaded, encrypted);
  loaded.crypt("CakeWallet");
  assert_eq!(loaded, seed);
}

#[test]
fn detects_every_single_bit_corruption() {
  let bytes: [u8; 32] = hex::decode(REFERENCE).unwrap().try_into().unwrap();
  for bit in 0 .. 256 {
    let mut corrupted = bytes;
    corrupted[bit / 8] ^= 1 << (bit % 8);
    Polyseed::load(&corrupted, Language::English).unwrap_err();
  }
  Polyseed::load(&[0; 32], Language::English).unwrap_err();
  Polyseed::load(&[255; 32], Language::English).unwrap_err();
}

#[test]
fn distinguishes_invalid_storage_layout_from_checksum_failures() {
  let bytes: [u8; 32] = hex::decode(REFERENCE).unwrap().try_into().unwrap();

  let corruptions: [fn(&mut [u8; 32]); 4] = [
    |value: &mut [u8; 32]| value[0] ^= 1,
    |value: &mut [u8; 32]| value[28] |= 0x80,
    |value: &mut [u8; 32]| value[29] ^= 1,
    |value: &mut [u8; 32]| value[31] ^= 0x80,
  ];
  for corrupt in corruptions {
    let mut corrupted = bytes;
    corrupt(&mut corrupted);
    assert_eq!(Polyseed::load(&corrupted, Language::English), Err(PolyseedError::InvalidFormat));
  }

  let mut bad_checksum = bytes;
  bad_checksum[30] ^= 1;
  assert_eq!(Polyseed::load(&bad_checksum, Language::English), Err(PolyseedError::InvalidChecksum));

  // A feature bit the checksum doesn't cover is corruption, not an unsupported feature
  let mut flipped_feature = bytes;
  flipped_feature[9] ^= 1 << 2;
  assert_eq!(
    Polyseed::load(&flipped_feature, Language::English),
    Err(PolyseedError::InvalidChecksum)
  );
}

#[test]
fn storage_is_independent_of_display_language() {
  let seed = Polyseed::from(Language::English, 0, 1_700_000_000, Zeroizing::new([0; 32])).unwrap();
  let korean = Polyseed::load(&seed.store(), Language::Korean).unwrap();
  assert_eq!(korean.store(), seed.store());
  assert_eq!(korean.key(), seed.key());
  assert_eq!(korean, Polyseed::from_string(Language::Korean, korean.to_string()).unwrap());
}
