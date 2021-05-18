pub mod setup;

pub trait Page {
    type Response;
    fn show(&mut self, ctx: &egui::CtxRef) -> Self::Response;
}
