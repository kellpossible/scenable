use directories::ProjectDirs;
use eyre::Context;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::view::page::setup::SetupParameters;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub xplane_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { xplane_dir: None }
    }
}

impl Settings {
    pub fn setup(&mut self, parameters: SetupParameters) {
        self.xplane_dir = Some(parameters.xplane_dir);
    }

    pub fn from_settings_file() -> eyre::Result<Option<Self>> {
        let settings_path = Self::settings_path()?;

        if !settings_path.exists() {
            tracing::debug!("settings file {:?} does not yet exist", &settings_path);
            return Ok(None);
        } else {
            tracing::debug!("Reading settings from {:?}", &settings_path);
            let file = std::fs::File::open(&settings_path).wrap_err_with(|| {
                eyre::eyre!("Unable to open settings file {:?}", &settings_path)
            })?;
            let settings: Self = ron::de::from_reader(file).wrap_err_with(|| {
                eyre::eyre!(
                    "Error while deserializing settings file {:?}",
                    &settings_path
                )
            })?;
            return Ok(Some(settings));
        }
    }

    pub fn save(&self) -> eyre::Result<()> {
        let settings_path = Self::settings_path()?;
        tracing::debug!("Writing settings to {:?}", &settings_path);
        let parent = settings_path.parent().ok_or_else(|| {
            eyre::eyre!(
                "Settings file does not have a parent directory: {:?}",
                &settings_path
            )
        })?;

        if !parent.exists() {
            std::fs::create_dir(parent)?;
        }

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(settings_path)?;

        ron::ser::to_writer_pretty(file, self, Default::default())
            .wrap_err("Error while serializing settings")?;

        Ok(())
    }

    fn settings_path() -> eyre::Result<PathBuf> {
        Ok(Self::project_dirs()?.config_dir().join("settings.ron"))
    }

    pub fn project_dirs() -> eyre::Result<ProjectDirs> {
        ProjectDirs::from("com", "lukefrisken", "scenable").ok_or_else(|| {
            eyre::eyre!("Unable to calculate configuration directory for your system.")
        })
    }
}
