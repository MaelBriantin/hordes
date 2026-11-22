//! The game's texts: every line the player reads is a message of `i18n/en/hordes.ftl`, in the
//! Fluent format (<https://projectfluent.org>), built into the game and looked up with `fl!`.
//! Another language is one more folder (`i18n/fr/`), which words each message its own way
//! without a change to the code; what it lacks shows in English. Same as termquest's.

use std::borrow::Cow;
use std::sync::LazyLock;

use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::unic_langid::LanguageIdentifier;
use i18n_embed::{I18nAssets, LanguageLoader};
use rust_embed::RustEmbed;

/// The texts' files, built into the game: `i18n/<language>/*.ftl`.
#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Files;

/// A language's files read as one: asked for `en/hordes.ftl`, it gives every `.ftl` file of
/// `en/` joined, should the texts one day be split into several files. No message may be defined
/// twice: a test checks it.
struct Texts;

impl I18nAssets for Texts {
    fn get_files(&self, file_path: &str) -> Vec<Cow<'_, [u8]>> {
        let Some((language, _)) = file_path.split_once('/') else {
            return Vec::new();
        };
        let folder = format!("{language}/");
        let mut joined = Vec::new();
        for name in Files::iter().filter(|name| name.starts_with(&folder) && name.ends_with(".ftl"))
        {
            if let Some(file) = Files::get(&name) {
                joined.extend_from_slice(&file.data);
                joined.push(b'\n');
            }
        }
        if joined.is_empty() {
            Vec::new()
        } else {
            vec![Cow::Owned(joined)]
        }
    }

    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(Files::iter().map(|name| name.to_string()))
    }
}

/// Where `fl!` looks its messages up: English, until `use_language` says otherwise.
pub static LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    // `fl!` checks at compile time that `hordes.ftl` holds every message it asks for.
    let english = loader.fallback_language().clone();
    load(&loader, english).expect("the English texts are built into the game");
    loader
});

/// Loads `language` into `loader`, English behind it for the messages it lacks.
fn load(
    loader: &FluentLanguageLoader,
    language: LanguageIdentifier,
) -> Result<(), i18n_embed::I18nEmbedError> {
    loader.load_languages(&Texts, &[language])?;
    // No invisible marks around the values: the terminal would count them as columns. Loading
    // brings them back, so it goes after each load.
    loader.set_use_isolating(false);
    Ok(())
}

/// Speaks `language` (`fr`): its messages first, then English for any it lacks. An unknown
/// language leaves English.
pub fn use_language(language: &str) {
    let Ok(language) = language.parse::<LanguageIdentifier>() else {
        return;
    };
    // A language that fails to load leaves the one on screen.
    let _ = load(&LOADER, language);
}

/// A message of the texts, by its id, with its values: `fl!("pause-quit")`,
/// `fl!("over-score", level = 4, kills = 37)`. It doesn't compile if the English file lacks the
/// message or one of its values.
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LOADER, $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LOADER, $message_id, $($args),*)
    }};
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn in_french_what_is_locked_reads_locked_and_what_is_not_unlocked() {
        let file = Files::get("fr/hordes.ftl").expect("the French file");
        let text = String::from_utf8_lossy(&file.data).into_owned();
        assert!(text.contains("\ncollection-locked = Verrouillé\n"));
        assert!(text.contains("\ncollection-unlocked = Déverrouillé\n"));
    }

    #[test]
    fn no_message_is_defined_twice_across_the_files() {
        let mut seen = HashSet::new();
        for name in Files::iter().filter(|name| name.ends_with(".ftl")) {
            let file = Files::get(&name).expect("an embedded file");
            let text = String::from_utf8_lossy(&file.data).into_owned();
            let language = name.split('/').next().unwrap_or_default().to_string();
            // A message starts a line with its id, then ` =`.
            for id in text
                .lines()
                .filter_map(|line| line.split_once(" =").map(|(id, _)| id))
            {
                if id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    assert!(
                        seen.insert((language.clone(), id.to_string())),
                        "`{id}` is defined twice in {language}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_translation_only_holds_messages_english_has() {
        let ids = |language: &str| -> HashSet<String> {
            let folder = format!("{language}/");
            Files::iter()
                .filter(|name| name.starts_with(&folder) && name.ends_with(".ftl"))
                .flat_map(|name| {
                    let file = Files::get(&name).expect("an embedded file");
                    message_ids(&String::from_utf8_lossy(&file.data))
                })
                .collect()
        };
        let english = ids("en");
        // Every other folder of `i18n/` is a translation.
        let translations: HashSet<String> = Files::iter()
            .filter_map(|name| {
                name.split_once('/')
                    .map(|(language, _)| language.to_string())
            })
            .filter(|language| language != "en")
            .collect();
        assert!(!translations.is_empty(), "no translation to check");
        for language in translations {
            let unknown: Vec<String> = ids(&language).difference(&english).cloned().collect();
            assert!(
                unknown.is_empty(),
                "{language} has ids English lacks: {unknown:?}"
            );
        }
    }

    #[test]
    fn french_has_its_own_words() {
        // A loader of its own: the game's is shared by every test.
        let loader = fluent_language_loader!();
        load(&loader, "fr".parse().expect("a language id")).expect("the French texts");

        assert_eq!(loader.get("pause-quit"), "Quitter");
        // Another language loaded, still no invisible marks around the values.
        let line = i18n_embed_fl::fl!(loader, "over-score", level = 4, kills = 37);
        assert_eq!(line, "Niveau 4 · 37 tués");
    }

    /// The ids of the messages a file defines: a message starts a line with its id, then ` =`.
    fn message_ids(text: &str) -> Vec<String> {
        text.lines()
            .filter_map(|line| line.split_once(" =").map(|(id, _)| id))
            .filter(|id| {
                !id.is_empty()
                    && id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            })
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn a_message_takes_its_values_without_invisible_marks() {
        let line = fl!("over-score", level = 4, kills = 37);

        assert_eq!(line, "Level 4 · 37 kills");
    }
}
