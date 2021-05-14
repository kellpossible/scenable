use std::sync::Arc;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, DesktopLanguageRequester, LanguageRequester, Localizer,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

lazy_static! {
    pub static ref LANGUAGE_LOADER: FluentLanguageLoader = fluent_language_loader!();
}

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

pub struct I18nGuard<'r> {
    _localizer: Arc<dyn Localizer>,
    _requester: DesktopLanguageRequester<'r>,
}

pub fn setup_i18n<'r>() -> eyre::Result<I18nGuard<'r>> {
    let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations);
    let mut language_requester = DesktopLanguageRequester::new();
    let localizer_arc: Arc<dyn Localizer> = Arc::new(localizer);
    language_requester.add_listener(Arc::downgrade(&localizer_arc));

    language_requester.poll()?;

    Ok(I18nGuard {
        _localizer: localizer_arc,
        _requester: language_requester,
    })
}
