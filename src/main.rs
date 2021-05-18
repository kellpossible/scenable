mod app;
mod i18n;
mod parsers;
mod settings;
mod view;

use app::ScenableApp;
use tracing_subscriber::EnvFilter;

use std::convert::TryFrom;

use crate::i18n::setup_i18n;

fn setup_reporting() {
    let env_filter: EnvFilter = match std::env::var("RUST_LOG") {
        Ok(env) => match EnvFilter::try_from(env) {
            Ok(filter) => Some(filter),
            Err(err) => {
                eprintln!("Unable to parse `RUST_LOG` envirnoment variable: {}.", err);
                None
            }
        },
        Err(_) => None,
    }
    .unwrap_or_else(|| EnvFilter::try_from("info").unwrap());

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("error installing tracing subscriber");
    color_eyre::install().expect("error installing color-eyre handlers");
}

fn main() -> eyre::Result<()> {
    setup_reporting();
    let _i18n_guard = setup_i18n()?;

    let native_options = epi::NativeOptions::default();
    let app = Box::new(ScenableApp::default());
    egui_glium::run(app, native_options);
}
