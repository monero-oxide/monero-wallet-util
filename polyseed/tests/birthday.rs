//! Host-supplied birthdays survive mnemonic and entropy reconstruction.

use polyseed::{Language, Polyseed};
use rand_core::OsRng;

#[test]
fn explicit_birthday_is_encoded_without_consulting_the_clock() {
  let epoch = 1_635_768_000;
  let step = 2_629_746;
  for (timestamp, expected) in [
    (0, epoch),
    (epoch - 1, epoch),
    (epoch + (22 * step) + 123, epoch + (22 * step)),
    (epoch + (1024 * step), epoch),
  ] {
    let seed = Polyseed::new_with_birthday(&mut OsRng, Language::English, timestamp);
    assert_eq!(seed.birthday(), expected);
    assert_eq!(seed, Polyseed::from_string(Language::English, seed.to_string()).unwrap());
    assert_eq!(
      seed,
      Polyseed::from(Language::English, 0, timestamp, seed.entropy().clone()).unwrap()
    );
  }
}
