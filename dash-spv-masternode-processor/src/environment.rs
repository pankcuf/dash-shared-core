// use bip39::Language;
// use ring::rand::SystemRandom;
// use crate::resource::bundle::Bundle;

use std::env;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub enum Language {
    #[default]
    English,
    Chinese,
    French,
    Italian,
    Japanese,
    Korean,
    Spanish
}

#[derive(Debug)]
pub struct Environment {
    // pub system_random: SystemRandom,
    // pub resource_bundle: Bundle,
    pub language: Language,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            // system_random: SystemRandom::new(),
            // resource_bundle: Bundle::default(),
            language: Language::English
        }
    }
}

// impl<'a> Default for &'a Environment {
//     fn default() -> Self {
//         &Environment::new_const_default()
//     }
// }

impl Environment {
    pub const DOMAIN: &'static str = "group.org.dashfoundation.dash-spv";
    pub fn cargo_home_dir() -> String {
        env::var("HOME").expect("No HOME defined")
    }
    pub fn cargo_manifest_dir() -> String {
        env::var("CARGO_MANIFEST_DIR").expect("No CARGO_MANIFEST_DIR defined")
    }
    pub fn cargo_current_dir() -> PathBuf {
        env::current_dir().expect("No current_dif")
    }


    pub const fn new_const_default() -> Self {
        Self { language: Language::English }
    }
    pub fn new(language: Language) -> Self {
        Self {
            language,
            // resource_bundle: Bundle {},
            // system_random: SystemRandom::new()

        }
    }

    pub fn mnemonic_from_phrase<T: bip0039::Language>(&self, phrase: &str) -> Result<bip0039::Mnemonic<T>, bip0039::Error> {
        match self.language {
            Language::English => bip0039::Mnemonic::<T>::from_phrase(phrase),
            Language::Chinese => bip0039::Mnemonic::<T>::from_phrase(phrase),
            Language::French => bip0039::Mnemonic::<T>::from_phrase(phrase),
            Language::Italian => bip0039::Mnemonic::<T>::from_phrase(phrase),
            Language::Japanese => bip0039::Mnemonic::<T>::from_phrase(phrase),
            Language::Korean => bip0039::Mnemonic::<T>::from_phrase(phrase),
            Language::Spanish => bip0039::Mnemonic::<T>::from_phrase(phrase),
        }
    }
    // bip0039::Mnemonic::<bip0039::English>::from_phrase(seed_phrase)
    // true if this is a "watch only" wallet with no signing ability
    pub fn watch_only() -> bool {
        false
    }

    // pub fn load_words_from_bundle(&self) -> Vec<String> {
    //     self.resource_bundle.load_words()
    // }
}
