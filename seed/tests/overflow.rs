//! Reject word triples whose decoded 32-bit value overflows `u32`.

use monero_seed::{Language, Seed, SeedError};
use zeroize::Zeroizing;

#[test]
fn rejects_out_of_range_word_triple_without_panicking() {
  // With w2 = "abducts" (index 1) and w3 = "abbey" (index 0), the third term is
  // 1625 * 1626^2 = 4_296_298_500 > u32::MAX, with a valid checksum word.
  let crafted_checksummed = "hemlock abducts abbey hacksaw boil superior inroads epoxy exhale \
                             orders cavernous second brunt saved richly lower upgrade hitched \
                             launching deepest mostly playful layout lower hemlock";
  assert_eq!(
    Seed::from_string(Language::English, Zeroizing::new(crafted_checksummed.to_owned())),
    Err(SeedError::InvalidSeed)
  );

  // Also reject the 24-word unchecksummed phrase and a triple in the final block.
  let unchecksummed = crafted_checksummed.rsplit_once(' ').unwrap().0;
  assert_eq!(
    Seed::from_string(Language::English, Zeroizing::new(unchecksummed.to_owned())),
    Err(SeedError::InvalidSeed)
  );

  let tail_overflow = format!("{} abbey abducts abbey", ["abbey"; 21].join(" "));
  assert_eq!(
    Seed::from_string(Language::English, Zeroizing::new(tail_overflow)),
    Err(SeedError::InvalidSeed)
  );
}
