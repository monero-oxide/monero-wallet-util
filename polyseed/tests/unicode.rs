//! Equivalent Unicode representations must recover the same seed.

use polyseed::{Language, Polyseed};
use unicode_normalization::UnicodeNormalization as _;
use zeroize::Zeroizing;

#[test]
fn spanish_reference_phrase_accepts_composed_and_decomposed_accents() {
  let phrase = "eje fin parte célebre tabú pestaña lienzo puma \
    prisión hora regalo lengua existir lápiz lote sonoro";
  let composed = Polyseed::from_string(Language::Spanish, Zeroizing::new(phrase.into())).unwrap();
  let decomposed =
    Polyseed::from_string(Language::Spanish, Zeroizing::new(phrase.nfkd().collect())).unwrap();
  assert_eq!(composed, decomposed);
  assert_eq!(
    hex::encode(composed.entropy().as_slice()),
    "5a2b02df7db21fcbe6ec6df137d54c7b20fd2b00000000000000000000000000"
  );
}

#[test]
fn language_output_is_composed_and_roundtrips() {
  for language in [Language::French, Language::Spanish, Language::Japanese, Language::Korean] {
    let seed = Polyseed::from(language, 0, 1_700_000_000, Zeroizing::new([0; 32])).unwrap();
    let phrase = seed.to_string();
    assert!(phrase.chars().eq(phrase.nfc()));
    assert_eq!(seed, Polyseed::from_string(language, phrase.clone()).unwrap());
    assert_eq!(
      seed,
      Polyseed::from_string(language, Zeroizing::new(phrase.nfkd().collect())).unwrap()
    );
    if language == Language::Japanese {
      assert_eq!(phrase.matches('\u{3000}').count(), 15);
    }
  }
}
