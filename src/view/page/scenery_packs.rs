use std::path::{Path, PathBuf};
use std::rc::Rc;

use egui::{Button, ScrollArea};

use crate::parsers::inifile::ToIniFile;
use crate::parsers::scenery_packs::{scenery_packs_ini, SceneryPacksIni};
use crate::state::ActionHistory;
use crate::state::UpdateSceneryPack;
use crate::state::UpdateSceneryPacks;
use crate::state::{ScenableAction, ScenableStateRef};

use super::Page;

pub struct SceneryPacksPage {
    state: ScenableStateRef,
    file_state_id: u64,
}

impl SceneryPacksPage {
    pub fn new(state: ScenableStateRef) -> Self {
        let mut new_self = Self {
            state,
            file_state_id: 0,
        };

        if let Err(error) = new_self.read_scenery_packs(true) {
            tracing::error!("Error while reading scenery packs: {}", error);
        }

        new_self
    }

    fn current_history_id(&self) -> u64 {
        let state = self.state.state();
        let (current_history, _) = state.scenery_packs_history.peek_current();
        current_history.item.id
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

        self.file_state_id = self.current_history_id();

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
                history: ActionHistory::Some(Rc::new("Read scenery_packs.ini")),
                reset_history,
            }));

        self.file_state_id = self.current_history_id();

        Ok(())
    }
}

impl Page for SceneryPacksPage {
    type Response = ();
    fn show(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) -> Self::Response {
        let current_state = self.state.state();
        egui::CentralPanel::default().show(ctx, |ui| {
            let scroll_area = ScrollArea::auto_sized();
            scroll_area.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let prev_history = current_state.scenery_packs_history.peek_prev();
                        let response = ui.add(Button::new("⮪").enabled(prev_history.is_some()));
                        let response = if let Some(_) = prev_history {
                            let (current_history, _) =
                                current_state.scenery_packs_history.peek_current();
                            response
                                // TODO: localize
                                .on_hover_text(format!("Undo: {}", &current_history.label))
                        } else {
                            response
                        };
                        if response.clicked() {
                            self.state.dispatch(ScenableAction::UndoSceneryPacks);
                        }

                        let next_history = current_state.scenery_packs_history.peek_next();
                        let response = ui.add(Button::new("⮫").enabled(next_history.is_some()));
                        let response = if let Some((next_history, _)) = next_history {
                            response
                                // TODO: localize
                                .on_hover_text(format!("Redo: {}", &next_history.label))
                        } else {
                            response
                        };
                        if response.clicked() {
                            self.state.dispatch(ScenableAction::RedoSceneryPacks);
                        }

                        // TODO: localize
                        let response = ui
                            .add(
                                Button::new("💾")
                                    .enabled(self.file_state_id != self.current_history_id()),
                            )
                            .on_hover_text("Save changes");

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

                                    let history_label = if new_scenery_pack.enabled {
                                        // TODO: localize
                                        Rc::new(format!(
                                            "Scenery pack {:?} enabled",
                                            scenery_pack.path
                                        ))
                                    } else {
                                        Rc::new(format!(
                                            "Scenery pack {:?} disabled",
                                            scenery_pack.path
                                        ))
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
