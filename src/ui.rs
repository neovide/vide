use bevy_reflect::Struct;
use egui::widgets::*;

use shader::model::ModelConstants;

pub struct UI {
    pub model_constants: ModelConstants,
}

impl Default for UI {
    fn default() -> Self {
        Self {
            model_constants: Default::default(),
        }
    }
}

impl epi::App for UI {
    fn name(&self) -> &str {
        "CAD UI"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::Window::new("Model Constants").resizable(true).show(ctx, |ui| {
            let field_count = self.model_constants.iter_fields().count();
            for i in 0..field_count {
                let field_name = self.model_constants.name_at(i).map(|s| s.to_string()).unwrap();
                let field_value = self.model_constants.field_at_mut(i).unwrap();
                if let Some(field_value) = field_value.downcast_mut::<f32>() {
                    ui.add(Label::new(format!("{}:", field_name)));
                    ui.add(DragValue::new(field_value).speed(0.01));
                }
            }
        });
    }
}
