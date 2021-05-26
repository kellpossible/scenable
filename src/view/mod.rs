pub mod page;

pub trait View {
    type Response;

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame<'_>) -> Self::Response;
}
