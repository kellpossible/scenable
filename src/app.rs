use crate::fl;
use iced::{
    text_input::{self, TextInput},
    Application, Checkbox, Column, Command, HorizontalAlignment, Length, Row, Text,
};

#[derive(Debug, Clone, Copy)]
pub enum AppMsg {
    Quit,
    Checked(bool),
    Nothing,
}

pub struct ScenableApp {
    checked: bool,
    directory_input_state: text_input::State,
}

impl Application for ScenableApp {
    type Executor = iced::executor::Default;
    type Message = AppMsg;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let app = ScenableApp {
            checked: false,
            directory_input_state: Default::default(),
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        fl!("window-title")
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        match message {
            AppMsg::Quit => {
                tracing::info!("Quitting");
                Command::none()
            }
            AppMsg::Checked(checked) => {
                self.checked = !self.checked;
                tracing::info!("Checked: {}", checked);
                Command::none()
            }
            AppMsg::Nothing => Command::none(),
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let checkbox = Checkbox::new(
            self.checked,
            fl!("scenery-enabled-checkbox-label"),
            AppMsg::Checked,
        );
        let directory = TextInput::new(&mut self.directory_input_state, "", "example_dir", |_| {
            AppMsg::Nothing
        });

        let row = Row::new().push(checkbox).push(directory);

        let content = Column::new().push(row);

        content.into()
    }
}
