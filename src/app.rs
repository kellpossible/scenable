use std::rc::Rc;

use crate::{
    fl,
    settings::Settings,
    state::{ScenableAction, ScenableReducer, ScenableState, ScenableStateRef},
    view::page::{
        scenery_packs::SceneryPacksPage,
        setup::{SetupPage, SetupPageResult},
        Page,
    },
};
use epi::App;
use reactive_state::middleware::simple_logger::SimpleLoggerMiddleware;
enum ScenablePage {
    Setup(SetupPage),
    SceneryPacks(SceneryPacksPage),
}

pub struct ScenableApp {
    name: String,
    state: ScenableStateRef,
    page: ScenablePage,
}

impl Default for ScenableApp {
    fn default() -> Self {
        let (setup_required, settings) = match Settings::from_settings_file() {
            Ok(None) => (true, Default::default()),
            Ok(Some(settings)) => (false, settings),
            Err(error) => {
                tracing::error!("Error reading settings from settings file: {}", error);
                (true, Default::default())
            }
        };

        let state = ScenableState {
            settings: Rc::new(settings),
            ..Default::default()
        };

        let state = ScenableStateRef::new(ScenableReducer, state);
        let log_middleware = SimpleLoggerMiddleware::new();
        state.add_middleware(log_middleware);

        let page = if setup_required {
            ScenablePage::Setup(Default::default())
        } else {
            ScenablePage::SceneryPacks(SceneryPacksPage::new(state.clone()))
        };

        Self {
            name: fl!("window-title"),
            state,
            page,
        }
    }
}

impl App for ScenableApp {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        match &mut self.page {
            ScenablePage::Setup(page) => {
                if let SetupPageResult::Continue(parameters) = page.show(ctx, frame).inner {
                    let mut settings = (*self.state.state().settings).clone();
                    settings.setup(parameters);
                    if let Err(error) = settings.save() {
                        tracing::error!("Error while saving settings: {}", error);
                    }
                    self.state
                        .dispatch(ScenableAction::UpdateSettings(settings));
                    self.page =
                        ScenablePage::SceneryPacks(SceneryPacksPage::new(self.state.clone()));
                }
            }
            ScenablePage::SceneryPacks(page) => page.show(ctx, frame),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
