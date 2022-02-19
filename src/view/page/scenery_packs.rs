use super::Page;
use crate::{
    fl,
    i18n::LocalizedString,
    parsers::{
        inifile::ToIniFile,
        scenery_packs::{scenery_packs_ini, SceneryPacksIni},
    },
    state::{
        ActionHistory, ScenableAction, ScenableStateRef, UpdateSceneryPack, UpdateSceneryPacks,
    },
};

use egui::{Button, ScrollArea};

use std::path::{Path, PathBuf};

pub struct SceneryPacksPage {
    state: ScenableStateRef,
}

impl SceneryPacksPage {
    pub fn new(state: ScenableStateRef) -> Self {
        let mut new_self = Self { state };

        if let Err(error) = new_self.read_scenery_packs(true) {
            tracing::error!("Error while reading scenery packs: {}", error);
        }

        new_self
    }

    fn scenery_packs_ini_path(&self) -> eyre::Result<PathBuf> {
        let state = self.state.state();
        let xplane_dir: &PathBuf = state.settings.xplane_dir.as_ref().ok_or_else(|| {
            eyre::eyre!("xplane_dir needs to be set before reading scenery packs")
        })?;

        Ok(xplane_dir.join("Custom Scenery").join("scenery_packs.ini"))
    }

    fn save_scenery_packs(&mut self) -> eyre::Result<()> {
        let ini_path = self.scenery_packs_ini_path()?;
        let ini = SceneryPacksIni {
            version: 1000,
            scenery_packs: self
                .state
                .state()
                .scenery_packs
                .clone()
                .into_iter()
                .collect(),
        };
        write_scenery_packs_ini(&ini, ini_path)?;

        self.state
            .dispatch(ScenableAction::UpdateSceneryPacksSyncStatus);

        Ok(())
    }

    /// Read scenery packs ini file, and replace what is the current
    /// state of scenery packs.
    fn read_scenery_packs(&mut self, reset_history: bool) -> eyre::Result<()> {
        let ini_path = self.scenery_packs_ini_path()?;
        let ini = read_scenery_packs_ini(ini_path)?;
        self.state
            .dispatch(ScenableAction::UpdateSceneryPacks(UpdateSceneryPacks {
                scenery_packs: im_rc::Vector::from(ini.scenery_packs),
                history: ActionHistory::Some(LocalizedString::from("Read scenery_packs.ini")),
                reset_history,
            }));

        self.state
            .dispatch(ScenableAction::UpdateSceneryPacksSyncStatus);

        Ok(())
    }
}

impl Page for SceneryPacksPage {
    type Response = ();
    fn show(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) -> Self::Response {
        let current_state = self.state.state();
        egui::CentralPanel::default().show(ctx, |ui| {
            let scroll_area = ScrollArea::vertical().auto_shrink([false, false]).max_width(f32::INFINITY);
            scroll_area.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let prev_history = current_state.scenery_packs_history.peek_prev();
                        let response = ui.add_enabled(prev_history.is_some(), Button::new("⮪"));
                        let response = if let Some(_) = prev_history {
                            let (current_history, _) =
                                current_state.scenery_packs_history.peek_current();
                            response.on_hover_text(fl!(
                                "undo-hover-text",
                                operation = current_history.label.to_string()
                            ))
                        } else {
                            response
                        };
                        if response.clicked() {
                            self.state.dispatch(ScenableAction::UndoSceneryPacks);
                        }

                        let next_history = current_state.scenery_packs_history.peek_next();
                        let response = ui.add_enabled(next_history.is_some(), Button::new("⮫"));
                        let response = if let Some((next_history, _)) = next_history {
                            response.on_hover_text(fl!(
                                "redo-hover-text",
                                operation = next_history.label.to_string()
                            ))
                        } else {
                            response
                        };
                        if response.clicked() {
                            self.state.dispatch(ScenableAction::RedoSceneryPacks);
                        }

                        let response = ui
                            .add_enabled(
                                !current_state.scenery_packs_synchronized(),
                                Button::new("💾")
                            )
                            .on_hover_text(fl!("save-hover-text"));

                        if response.clicked() {
                            if let Err(error) = self.save_scenery_packs() {
                                tracing::error!("Error saving scenery_packs.ini: {}", error);
                            }
                        }

                        ()
                    });

                    current_state.scenery_packs.iter().enumerate().for_each(
                        |(index, scenery_pack)| {
                            ui.horizontal(|ui| {
                                let mut enabled = scenery_pack.enabled;
                                if ui.checkbox(&mut enabled, "").clicked() {
                                    let mut new_scenery_pack = scenery_pack.clone();
                                    new_scenery_pack.enabled = enabled;

                                    let path_debug = format!("{:?}", scenery_pack.path);
                                    let history_label = if new_scenery_pack.enabled {
                                        LocalizedString::new(move || {
                                            fl!(
                                                "scenery-pack-enabled-operation",
                                                path = path_debug.clone()
                                            )
                                        })
                                    } else {
                                        LocalizedString::new(move || {
                                            fl!(
                                                "scenery-pack-enabled-operation",
                                                path = path_debug.clone()
                                            )
                                        })
                                    };

                                    self.state.dispatch(ScenableAction::UpdateSceneryPack(
                                        UpdateSceneryPack {
                                            index,
                                            scenery_pack: new_scenery_pack,
                                            history: ActionHistory::Some(history_label),
                                        },
                                    ))
                                }
                                ui.label(scenery_pack.path.to_string_lossy().to_string());
                            });
                        },
                    )
                })
            })
        });
    }
}

fn read_scenery_packs_ini(ini_path: impl AsRef<Path>) -> eyre::Result<SceneryPacksIni> {
    tracing::info!("Reading scenery packs from {:?}", ini_path.as_ref());
    let ini_file_string = std::fs::read_to_string(&ini_path)?;
    let (_, ini) = scenery_packs_ini(&ini_file_string)
        .map_err(|error| eyre::eyre!("Error parsing {:?}: {}", ini_path.as_ref(), error))?;
    Ok(ini.clone())
}

fn write_scenery_packs_ini(ini: &SceneryPacksIni, ini_path: impl AsRef<Path>) -> eyre::Result<()> {
    tracing::info!("Writing scenery packs to {:?}", ini_path.as_ref());
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(ini_path)?;
    ini.write_ini(&mut file)?;
    Ok(())
}
