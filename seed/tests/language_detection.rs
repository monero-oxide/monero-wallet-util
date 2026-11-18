//! Automatic language detection for Monero classic seeds.

use monero_seed::{Language, Seed, SeedError};
use zeroize::Zeroizing;

#[test]
fn detects_all_languages_and_canonical_ambiguity_order() {
  let cases = [
    (
      Language::Chinese,
      "摇 曲 艺 武 滴 然 效 似 赏 式 祥 歌 买 疑 小 碧 堆 博 键 房 鲜 悲 付 喷 武",
      "a5e4fff1706ef9212993a69f246f5c95ad6d84371692d63e9bb0ea112a58340d",
    ),
    (
      Language::English,
      "washing thirsty occur lectures tuesday fainted toxic adapt \
       abnormal memoir nylon mostly building shrugged online ember northern \
       ruby woes dauntless boil family illness inroads northern",
      "c0af65c0dd837e666b9d0dfed62745f4df35aed7ea619b2798a709f0fe545403",
    ),
    (
      Language::Dutch,
      "setwinst riphagen vimmetje extase blief tuitelig fuiven meifeest \
       ponywagen zesmaal ripdeal matverf codetaal leut ivoor rotten \
       wisgerhof winzucht typograaf atrium rein zilt traktaat verzaagd setwinst",
      "e2d2873085c447c2bc7664222ac8f7d240df3aeac137f5ff2022eaa629e5b10a",
    ),
    (
      Language::French,
      "poids vaseux tarte bazar poivre effet entier nuance \
       sensuel ennui pacte osselet poudre battre alibi mouton \
       stade paquet pliage gibier type question position projet pliage",
      "2dd39ff1a4628a94b5c2ec3e42fb3dfe15c2b2f010154dc3b3de6791e805b904",
    ),
    (
      Language::Spanish,
      "minero ocupar mirar evadir octubre cal logro miope \
       opaco disco ancla litio clase cuello nasal clase \
       fiar avance deseo mente grumo negro cordón croqueta clase",
      "ae2c9bebdddac067d73ec0180147fc92bdf9ac7337f1bcafbbe57dd13558eb02",
    ),
    (
      Language::German,
      "Kaliber Gabelung Tapir Liveband Favorit Specht Enklave Nabel \
       Jupiter Foliant Chronik nisten löten Vase Aussage Rekord \
       Yeti Gesetz Eleganz Alraune Künstler Almweide Jahr Kastanie Almweide",
      "79801b7a1b9796856e2397d862a113862e1fdc289a205e79d8d70995b276db06",
    ),
    (
      Language::Italian,
      "cavo pancetta auto fulmine alleanza filmato diavolo prato \
       forzare meritare litigare lezione segreto evasione votare buio \
       licenza cliente dorso natale crescere vento tutelare vetta evasione",
      "5e7fd774eb00fa5877e2a8b4dc9c7ffe111008a3891220b56a6e49ac816d650a",
    ),
    (
      Language::Portuguese,
      "agito eventualidade onus itrio holograma sodomizar objetos dobro \
       iugoslavo bcrepuscular odalisca abjeto iuane darwinista eczema acetona \
       cibernetico hoquei gleba driver buffer azoto megera nogueira agito",
      "13b3115f37e35c6aa1db97428b897e584698670c1b27854568d678e729200c0f",
    ),
    (
      Language::Japanese,
      "ぜんぶ どうぐ おたがい せんきょ おうじ そんちょう じゅしん いろえんぴつ \
       かほう つかれる えらぶ にちじょう くのう にちようび ぬまえび さんきゃく \
       おおや ちぬき うすめる いがく せつでん さうな すいえい せつだん おおや",
      "c56e895cdb13007eda8399222974cdbab493640663804b93cbef3d8c3df80b0b",
    ),
    (
      Language::Russian,
      "шатер икра нация ехать получать инерция доза реальный \
       рыжий таможня лопата душа веселый клетка атлас лекция \
       обгонять паек наивный лыжный дурак стать ежик задача паек",
      "7cb5492df5eb2db4c84af20766391cd3e3662ab1a241c70fc881f3d02c381f05",
    ),
    (
      Language::Esperanto,
      "ukazo klini peco etikedo fabriko imitado onklino urino \
       pudro incidento kumuluso ikono smirgi hirundo uretro krii \
       sparkado super speciala pupo alpinisto cvana vokegi zombio fabriko",
      "82ebf0336d3b152701964ed41df6b6e9a035e57fc98b84039ed0bd4611c58904",
    ),
    (
      Language::Lojban,
      "jetnu vensa julne xrotu xamsi julne cutci dakli \
       mlatu xedja muvgau palpi xindo sfubu ciste cinri \
       blabi darno dembi janli blabi fenki bukpu burcu blabi",
      "e4f8c6819ab6cf792cebb858caabac9307fd646901d72123e0367ebc0a79c200",
    ),
    (
      Language::DeprecatedEnglish,
      "glorious especially puff son moment add youth nowhere \
       throw glide grip wrong rhythm consume very swear \
       bitter heavy eventually begin reason flirt type unable",
      "647f4765b66b636ff07170ab6280a9a6804dfbaf19db2ad37d23be024a18730b",
    ),
  ];

  for (expected_lang, words, expected_spend) in cases {
    let seed = Seed::from_string_auto(Zeroizing::new(words.to_owned())).unwrap();
    assert_eq!(seed.language(), expected_lang);
    assert_eq!(hex::encode(seed.entropy().as_slice()), expected_spend);
  }

  // Words shared by English and Spanish must deterministically resolve to English
  // first, matching Monero's `find_seed_language`.
  let ambiguous = Zeroizing::new(["audio"; 25].join(" "));
  let detected = Seed::from_string_auto(ambiguous.clone()).unwrap();
  assert_eq!(detected.language(), Language::English);
  let explicit_spanish = Seed::from_string(Language::Spanish, ambiguous).unwrap();
  assert_eq!(explicit_spanish.language(), Language::Spanish);
  assert_ne!(detected.entropy(), explicit_spanish.entropy());
}

#[test]
fn detects_checksum_deprecated_english_and_invalid_seed_errors() {
  let bad_checksum = "washing thirsty occur lectures tuesday fainted toxic adapt \
                      abnormal memoir nylon mostly building shrugged online ember northern \
                      ruby woes dauntless boil family illness inroads abbey";
  assert_eq!(
    Seed::from_string_auto(Zeroizing::new(bad_checksum.to_owned())),
    Err(SeedError::InvalidChecksum)
  );

  let deprecated_with_checksum = "glorious especially puff son moment add youth nowhere \
                                  throw glide grip wrong rhythm consume very swear \
                                  bitter heavy eventually begin reason flirt type unable glorious";
  assert_eq!(
    Seed::from_string_auto(Zeroizing::new(deprecated_with_checksum.to_owned())),
    Err(SeedError::DeprecatedEnglishWithChecksum)
  );

  let unknown_25 = Zeroizing::new(["zzzzzzzz"; 25].join(" "));
  assert_eq!(Seed::from_string_auto(unknown_25), Err(SeedError::InvalidSeed));
  assert_eq!(
    Seed::from_string_auto(Zeroizing::new("short seed".to_owned())),
    Err(SeedError::InvalidSeed)
  );
}

#[test]
fn does_not_fall_through_to_a_later_language_than_monero_selects() {
  // A valid Spanish seed whose words, as prefixes, are also all English with a valid English
  // checksum. Monero selects English for it, where it is not a canonical scalar, so it must not
  // be silently recovered as the Spanish seed.
  let words = Zeroizing::new(
    "guiso pollo echar motivo rehén pepino colcha fianza obeso fax jarabe cero almeja \
     cochino afinar lamer cuneta borrar rescate nicho ligero pizca retorno alejar cochino"
      .to_owned(),
  );
  let spanish = Seed::from_string(Language::Spanish, words.clone()).unwrap();
  assert_eq!(
    hex::encode(spanish.entropy().as_slice()),
    "50af0b743e467ce5572797582c078aa7c20c63c4e5f26adc7d5c7aded34f0a0e"
  );
  assert_eq!(Seed::from_string(Language::English, words.clone()), Err(SeedError::InvalidSeed));
  assert_eq!(Seed::from_string_auto(words), Err(SeedError::InvalidSeed));
}
