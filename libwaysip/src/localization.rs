use i18n_embed::{
    DesktopLanguageRequester,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;
use std::sync::LazyLock;

// code was taken from lala-bar, simplified a bit to keep only the simple cases
#[derive(RustEmbed)]
#[folder = "assets/locales/"]
struct Asset;

// static instance of localizer for global use
pub static LOCALIZER: LazyLock<Localizer> = LazyLock::new(Localizer::new);

pub struct Localizer {
    pub loader: FluentLanguageLoader,
}

impl Localizer {
    pub fn new() -> Self {
        let loader: FluentLanguageLoader = fluent_language_loader!();

        let requested_languages = DesktopLanguageRequester::requested_languages();

        if let Err(e) = i18n_embed::select(&loader, &Asset, &requested_languages) {
            tracing::warn!(
                "Localized strings not found for system lang, falling back to English: {}",
                e
            );
        }

        Self { loader }
    }
}

#[macro_export]
macro_rules! fl {
    // for static strings
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::localization::LOCALIZER.loader, $message_id)
    }};

    //for cases with single arg
    ($message_id:literal, $key:ident = $value:expr $(, $rest:tt)*) => {{
        i18n_embed_fl::fl!($crate::localization::LOCALIZER.loader, $message_id, $key = $value $(, $rest)*)
    }};
}
