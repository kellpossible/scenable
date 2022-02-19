pub mod page;

pub trait View {
    type Response;

    fn ui(&mut self, ui: &mut egui::Ui, frame: &epi::Frame) -> Self::Response;
}
