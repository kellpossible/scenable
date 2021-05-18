use std::path::{Path, PathBuf};

use egui::{Color32, InnerResponse};
use im_native_dialog::ImNativeFileDialog;

use crate::{fl, view::View};

use super::Page;

#[derive(Default)]
pub struct SetupPage {
    xplane_dir: PathBuf,
    xplane_dir_dialog: ImNativeFileDialog<Option<PathBuf>>,
}

pub struct SetupParameters {
    pub xplane_dir: PathBuf,
}

pub enum SetupPageResult {
    Waiting,
    Continue(SetupParameters),
}

impl View for SetupPage {
    type Response = InnerResponse<SetupPageResult>;

    fn ui(&mut self, ui: &mut egui::Ui) -> Self::Response {
        if let Some(result) = self.xplane_dir_dialog.check() {
            match result {
                Ok(Some(path)) => self.xplane_dir = path,
                Ok(None) => {}
                Err(error) => {
                    tracing::error!("Error selecting xplane_dir: {}", error)
                }
            }
        }

        ui.vertical(|ui| {
            ui.heading(fl!("setup-title"));
            ui.horizontal(|ui| {
                ui.set_enabled(!self.xplane_dir_dialog.is_open());
                ui.label(fl!("xplane-dir-label"));
                // make the text edit expand to fill available space (with browse button on right)
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if ui.button(fl!("browse-directory-button-title")).clicked() {
                        self.xplane_dir_dialog
                            .open_single_dir(Some(self.xplane_dir.clone()))
                            .expect("Unable to open xplane_path dialog");
                    }

                    let text_edit_size = ui.available_size();

                    let text_original = self.xplane_dir.to_string_lossy().to_string();
                    let mut text_edit = text_original.clone();
                    ui.add_sized(text_edit_size, egui::TextEdit::singleline(&mut text_edit));

                    if text_edit != text_original {
                        self.xplane_dir = PathBuf::from(text_edit);
                    }
                })
            });

            let xplane_path_valid = if self.xplane_dir.as_os_str().is_empty() {
                ui.label(fl!("specify-xplane-dir-message"));
                false
            } else if !self.xplane_dir.exists() {
                ui.colored_label(Color32::RED, fl!("path-not-exist-error"));
                false
            } else if !self.xplane_dir.is_dir() {
                ui.colored_label(Color32::RED, fl!("path-not-dir-error"));
                false
            } else if !contains_xplane_executable(&self.xplane_dir) {
                ui.colored_label(Color32::RED, fl!("path-not-xplane-dir"));
                false
            } else {
                true
            };

            ui.set_enabled(xplane_path_valid);
            if ui.button("Continue").clicked() {
                SetupPageResult::Continue(SetupParameters {
                    xplane_dir: self.xplane_dir.clone(),
                })
            } else {
                SetupPageResult::Waiting
            }
        })
    }
}

impl Page for SetupPage {
    type Response = InnerResponse<SetupPageResult>;
    fn show(&mut self, ctx: &egui::CtxRef) -> Self::Response {
        egui::CentralPanel::default()
            .show(ctx, |ui| self.ui(ui))
            .inner
    }
}

fn contains_xplane_executable(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    #[cfg(target_family = "windows")]
    return path.join("X-Plane-x86_64.exe").exists();

    #[cfg(target_family = "unix")]
    return path.join("X-Plane-x86_64").exists();
}
