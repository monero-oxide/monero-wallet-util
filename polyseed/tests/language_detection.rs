//! Automatic detection requires exactly one language to decode the mnemonic.

use polyseed::{Language, Polyseed, PolyseedError};
use zeroize::Zeroizing;

#[test]
fn detects_unambiguous_languages() {
  for language in [Language::English, Language::Spanish, Language::Japanese, Language::Korean] {
    let seed = Polyseed::from(language, 0, 1_700_000_000, Zeroizing::new([0; 32])).unwrap();
    assert_eq!(seed.language(), language);
    assert_eq!(Polyseed::from_string_auto(seed.to_string()).unwrap(), (seed, language));
  }
}

#[test]
fn uses_the_checksum_to_select_between_languages_sharing_prefixes() {
  // From tevador/polyseed's tests. Every word is a prefix in both the English and Spanish
  // wordlists, yet the checksum is only valid for Spanish.
  let phrase = Zeroizing::new(
    "impo sort usua cabi venu nobl oliv clim cont barr marc auto prod vaca torn fati".to_owned(),
  );
  assert_eq!(
    Polyseed::from_string(Language::English, phrase.clone()),
    Err(PolyseedError::InvalidChecksum)
  );
  let (seed, language) = Polyseed::from_string_auto(phrase.clone()).unwrap();
  assert_eq!(language, Language::Spanish);
  assert_eq!(seed, Polyseed::from_string(Language::Spanish, phrase).unwrap());
}

#[test]
fn rejects_mnemonics_which_decode_in_multiple_languages() {
  let seed = Polyseed::from(Language::ChineseSimplified, 0, 0, Zeroizing::new([0; 32])).unwrap();
  let phrase = seed.to_string();
  Polyseed::from_string(Language::ChineseTraditional, phrase.clone()).unwrap();
  assert_eq!(Polyseed::from_string_auto(phrase), Err(PolyseedError::MultipleLanguagesMatch));
}

#[test]
fn reports_why_no_language_decoded_the_mnemonic() {
  let seed = Polyseed::from(Language::English, 0, 0, Zeroizing::new([0; 32])).unwrap();
  let phrase = seed.to_string();
  let words = phrase.split_whitespace().collect::<Vec<_>>();

  for count in [1, 15, 17] {
    let phrase = words.iter().copied().cycle().take(count).collect::<Vec<_>>().join(" ");
    assert_eq!(
      Polyseed::from_string_auto(Zeroizing::new(phrase)),
      Err(PolyseedError::InvalidWordCount)
    );
  }

  let mut bad_checksum = words.clone();
  bad_checksum[15] = if words[15] == "abandon" { "ability" } else { "abandon" };
  assert_eq!(
    Polyseed::from_string_auto(Zeroizing::new(bad_checksum.join(" "))),
    Err(PolyseedError::InvalidChecksum)
  );

  let mut unknown_word = words;
  unknown_word[3] = "zzzzzzzz";
  assert_eq!(
    Polyseed::from_string_auto(Zeroizing::new(unknown_word.join(" "))),
    Err(PolyseedError::InvalidSeed)
  );
}
