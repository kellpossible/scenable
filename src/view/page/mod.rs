pub mod scenery_packs;
pub mod setup;

/// A page, a widget which has complete control of the screen.
pub trait Page {
    type Response;
    fn show(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) -> Self::Response;
}
