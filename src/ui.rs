use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

#[derive(Resource)]
pub struct PollParams {
    pub radius: f32,
    pub height: f32,
    pub angle: f32,
    pub speed: f32,
    pub n: usize,
    pub orth: bool,
}

impl Default for PollParams {
    fn default() -> Self {
        Self {
            radius: 3.0,
            height: 4.0,
            angle: 45.,
            speed: 2.0,
            n: 10,
            orth: true,
        }
    }
}

pub fn ui(mut contexts: EguiContexts, mut params: ResMut<PollParams>) -> Result {
    egui::Window::new("Barber Poll").show(contexts.ctx_mut()?, |ui| {
        ui.add(egui::Slider::new(&mut params.radius, 0.1..=8.0).text("Radius"));
        ui.add(egui::Slider::new(&mut params.height, 0.1..=12.0).text("Height"));
        ui.add(egui::Slider::new(&mut params.angle, 0.0..=180.0).text("Angle"));
        ui.add(egui::Slider::new(&mut params.speed, 0.1..=5.0).text("Speed"));
        ui.add(egui::Slider::new(&mut params.n, 1..=20).text("N"));
        ui.add(egui::Checkbox::new(&mut params.orth, "Orthographic"));
    });
    Ok(())
}
