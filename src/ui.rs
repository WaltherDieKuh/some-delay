use std::sync::Arc;
use std::f32::consts::PI;
use eframe::egui;
use eframe::egui::{Pos2, Sense, Stroke, Color32, Vec2};

use crate::audio::AudioEngine;

pub struct DelayApp {
    engine: Option<Arc<AudioEngine>>,

    // parameters
    gain_db: f32,    // -60..12 (centered as before)
    delay_ms: f32,   // 1..2000
    feedback: f32,   // 0..0.95
    mix: f32,        // 0..1

    // per-knob dragging state
    drag_gain_y: f32,
    drag_gain_start: f32,
    dragging_gain: bool,

    drag_delay_y: f32,
    drag_delay_start: f32,
    dragging_delay: bool,

    drag_fb_y: f32,
    drag_fb_start: f32,
    dragging_fb: bool,

    drag_mix_y: f32,
    drag_mix_start: f32,
    dragging_mix: bool,
}

impl DelayApp {
    pub fn new(engine: Option<Arc<AudioEngine>>) -> Self {
        Self {
            engine,
            gain_db: 0.0,
            delay_ms: 500.0,
            feedback: 0.3,
            mix: 0.5,
            drag_gain_y: 0.0,
            drag_gain_start: 0.0,
            dragging_gain: false,
            drag_delay_y: 0.0,
            drag_delay_start: 0.0,
            dragging_delay: false,
            drag_fb_y: 0.0,
            drag_fb_start: 0.0,
            dragging_fb: false,
            drag_mix_y: 0.0,
            drag_mix_start: 0.0,
            dragging_mix: false,
        }
    }
}

// helper mappers
fn gain_value_to_frac(v: f32) -> f32 {
    if v >= 0.0 { (v / 12.0).clamp(0.0, 1.0) } else { (v / 60.0).clamp(-1.0, 0.0) }
}
fn gain_frac_to_value(f: f32) -> f32 {
    if f >= 0.0 { (f * 12.0).clamp(0.0, 12.0) } else { (f * 60.0).clamp(-60.0, 0.0) }
}
fn frac_linear_to_range(f: f32, min: f32, max: f32) -> f32 {
    let t = (f + 1.0) * 0.5; // 0..1
    min + t * (max - min)
}
fn range_to_frac_linear(v: f32, min: f32, max: f32) -> f32 {
    let t = ((v - min) / (max - min)).clamp(0.0, 1.0);
    t * 2.0 - 1.0
}

impl eframe::App for DelayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Delay Plugin");
            });

            ui.horizontal(|ui| {
                // make a small function to render a knob area and handle vertical drag; returns new frac
                let mut draw_knob = |label: &str,
                                     value_frac_start: &mut f32,
                                     dragging: &mut bool,
                                     drag_y: &mut f32,
                                     drag_start: &mut f32,
                                     min_val: f32,
                                     max_val: f32,
                                     value_to_frac: fn(f32)->f32,
                                     frac_to_value: fn(f32)->f32,
                                     on_change: Option<&dyn Fn(f32)>| {
                    ui.vertical(|ui| {
                        ui.label(label);
                        let size = Vec2::new(100.0, 100.0);
                        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
                        let center = rect.center();
                        let radius = rect.width().min(rect.height()) / 2.0 - 6.0;

                        let pointer_down = ui.input(|i| i.pointer.primary_down());
                        let input_pos = ui.input(|i| i.pointer.interact_pos());

                        if pointer_down && !*dragging {
                            if let Some(pos) = input_pos {
                                let dx = pos.x - center.x;
                                let dy = pos.y - center.y;
                                if dx*dx + dy*dy <= radius*radius {
                                    *dragging = true;
                                    *drag_y = pos.y;
                                    *drag_start = value_to_frac(frac_to_value(*value_frac_start));
                                }
                            }
                        }

                        if *dragging {
                            if pointer_down {
                                if let Some(pos) = input_pos {
                                    let dy = *drag_y - pos.y;
                                    let sensitivity = rect.height().max(1.0);
                                    let delta_frac = (dy / sensitivity).clamp(-1.0, 1.0);
                                    let new_frac = (*drag_start + delta_frac).clamp(-1.0, 1.0);
                                    *value_frac_start = range_to_frac_linear(frac_to_value(new_frac), min_val, max_val);
                                    let v = frac_to_value(new_frac);
                                    if let Some(cb) = on_change { cb(v); }
                                }
                            } else {
                                *dragging = false;
                            }
                        }

                        // draw knob using frac from value_frac_start
                        let frac = value_to_frac(frac_to_value(*value_frac_start));
                        // angle mapping
                        let min_angle = -3.0 * PI / 4.0;
                        let max_angle = 3.0 * PI / 4.0;
                        let angle_range = max_angle - min_angle;
                        let center_angle = min_angle + angle_range / 2.0;
                        let rot_offset = 3.0 * PI / 2.0;
                        let angle = center_angle + frac * (angle_range / 2.0) + rot_offset;

                        let painter = ui.painter();
                        painter.circle_filled(center, radius, Color32::from_rgb(30,30,30));
                        painter.circle_stroke(center, radius, Stroke::new(2.5, Color32::from_rgb(200,200,200)));
                        let tip = Pos2::new(center.x + angle.cos() * radius * 0.7, center.y + angle.sin() * radius * 0.7);
                        painter.line_segment([center, tip], Stroke::new(3.5, Color32::from_rgb(100,200,255)));

                        ui.label(format!("{:.2}", frac_to_value(value_to_frac(frac_to_value(*value_frac_start)))));
                    });
                };

                // Gain knob (special asymmetric mapping)
                let mut gain_frac = gain_value_to_frac(self.gain_db);
                let engine_gain = self.engine.clone();
                draw_knob(
                    "Gain (dB)",
                    &mut gain_frac,
                    &mut self.dragging_gain,
                    &mut self.drag_gain_y,
                    &mut self.drag_gain_start,
                    -60.0,
                    12.0,
                    |v| gain_value_to_frac(v),
                    |f| gain_frac_to_value(f),
                    Some(&|v: f32| { if let Some(e) = &engine_gain { e.set_gain_db(v); } }),
                );
                self.gain_db = gain_frac_to_value(gain_frac);

                // Delay time knob
                let mut delay_frac = range_to_frac_linear(self.delay_ms, 1.0, 2000.0);
                let engine_delay = self.engine.clone();
                draw_knob(
                    "Delay (ms)",
                    &mut delay_frac,
                    &mut self.dragging_delay,
                    &mut self.drag_delay_y,
                    &mut self.drag_delay_start,
                    1.0,
                    2000.0,
                    |v| range_to_frac_linear(v, 1.0, 2000.0),
                    |f| frac_linear_to_range(f, 1.0, 2000.0),
                    Some(&|v: f32| { if let Some(e) = &engine_delay { e.set_delay_ms(v); } }),
                );
                self.delay_ms = frac_linear_to_range(delay_frac, 1.0, 2000.0);

                // Feedback knob
                let mut fb_frac = range_to_frac_linear(self.feedback, 0.0, 0.95);
                let engine_fb = self.engine.clone();
                draw_knob(
                    "Feedback",
                    &mut fb_frac,
                    &mut self.dragging_fb,
                    &mut self.drag_fb_y,
                    &mut self.drag_fb_start,
                    0.0,
                    0.95,
                    |v| range_to_frac_linear(v, 0.0, 0.95),
                    |f| frac_linear_to_range(f, 0.0, 0.95),
                    Some(&|v: f32| { if let Some(e) = &engine_fb { e.set_feedback(v); } }),
                );
                self.feedback = frac_linear_to_range(fb_frac, 0.0, 0.95);

                // Mix knob
                let mut mix_frac = range_to_frac_linear(self.mix, 0.0, 1.0);
                let engine_mix = self.engine.clone();
                draw_knob(
                    "Mix",
                    &mut mix_frac,
                    &mut self.dragging_mix,
                    &mut self.drag_mix_y,
                    &mut self.drag_mix_start,
                    0.0,
                    1.0,
                    |v| range_to_frac_linear(v, 0.0, 1.0),
                    |f| frac_linear_to_range(f, 0.0, 1.0),
                    Some(&|v: f32| { if let Some(e) = &engine_mix { e.set_mix(v); } }),
                );
                self.mix = frac_linear_to_range(mix_frac, 0.0, 1.0);
            });
        });
    }
}

pub fn run(engine: Option<Arc<AudioEngine>>) -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Delay Plugin", options, Box::new(|_cc| Box::new(DelayApp::new(engine))));
    Ok(())
}
