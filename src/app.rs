use std::path::PathBuf;

use crate::{
    fl,
    settings::Settings,
    view::{
        page::{
            setup::{SetupPage, SetupPageResult},
            Page,
        },
        View,
    },
};
use epi::App;

enum MainPage {
    Setup(SetupPage),
    SceneryPacks,
}

pub struct ScenableApp {
    name: String,
    settings: Settings,
    page: MainPage,
}

impl Default for ScenableApp {
    fn default() -> Self {
        let (page, settings) = match Settings::from_settings_file() {
            Ok(None) => (MainPage::Setup(SetupPage::default()), Default::default()),
            Ok(Some(settings)) => (MainPage::SceneryPacks, settings),
            Err(error) => {
                tracing::error!("Error reading settings from settings file: {}", error);
                (MainPage::Setup(SetupPage::default()), Default::default())
            }
        };

        Self {
            name: fl!("window-title"),
            settings,
            page,
        }
    }
}

impl App for ScenableApp {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        match &mut self.page {
            MainPage::Setup(setup_page) => {
                if let SetupPageResult::Continue(parameters) = setup_page.show(ctx).inner {
                    self.settings.setup(parameters);
                    self.settings.save().unwrap();
                    self.page = MainPage::SceneryPacks;
                }
            }
            MainPage::SceneryPacks => {}
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
