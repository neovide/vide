pub struct UI {}

impl Default for UI {
    fn default() -> Self {
        Self {}
    }
}

impl epi::App for UI {
    fn name(&self) -> &str {
        "Rusty Cad UI"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::Window::new("Test Window").show(ctx, |ui| {
            ui.heading("This is a test window");
        });
    }
}
