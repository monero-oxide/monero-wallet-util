//! Invalid mnemonic lengths must be rejected before indexing the polynomial.

use polyseed::{Language, Polyseed, PolyseedError};
use zeroize::Zeroizing;

#[test]
fn rejects_incorrect_word_counts_without_panicking() {
  for count in [0, 1, 15, 17, 32] {
    let phrase = core::iter::repeat("abandon").take(count).collect::<Vec<_>>().join(" ");
    assert_eq!(
      Polyseed::from_string(Language::English, Zeroizing::new(phrase)),
      Err(PolyseedError::InvalidWordCount)
    );
  }
  assert_eq!(
    Polyseed::from_string(Language::English, Zeroizing::new(" \t\n ".into())),
    Err(PolyseedError::InvalidWordCount)
  );
}
