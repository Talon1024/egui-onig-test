use eframe::egui::{self, TextFormat, Color32, Rgba, FontId};
use onig::Regex;
use std::{
	f32::consts::{TAU},
};
use egui::text::LayoutJob;
pub mod caps;
use caps::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Call this once from the HTML.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
	eframe::start_web(canvas_id, Box::new(|cc| Box::new(MyEguiApp::new(cc))))
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
	let native_options = eframe::NativeOptions::default();
	eframe::run_native("Oniguruma regex tester", native_options,
		Box::new(|cc| Box::new(MyEguiApp::new(cc))));
}

const HUE_MAX: f32 = 12.;
const HUE_MIN: f32 = (1. / HUE_MAX) * TAU;

fn hue_to_rgb(hue: f32) -> Color32 {
	let subtract: [f32; 3] = [0., 0.333333333, 0.666666666];
	let rgb = subtract.map(|sub| {
		((hue - sub * TAU).cos() + 0.5).clamp(0., 1.)
	});
	Color32::from(Rgba::from_rgb(rgb[0], rgb[1], rgb[2]))
}

#[derive(Default)]
struct MyEguiApp {
	regex_str: String,
	regex: Option<Regex>,
	regex_error: String,
	test_text: String,
	test_captures: Option<Vec<CaptureInfo>>,
	test_text_monospace: bool, // TODO: Make this a FontId
	test_text_size: f32,
}

impl MyEguiApp {
	fn new(_cc: &eframe::CreationContext<'_>) -> Self {
		// Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
		// Restore app state using cc.storage (requires the "persistence" feature).
		// Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
		// for e.g. egui::PaintCallback.
		Self {
			test_text_size: 14.,
			..Default::default()
		}
	}
}

impl eframe::App for MyEguiApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

		// Update whenever regex or test_text changes
		let update_text = |regex: Option<&Regex>, test_text: &str| {
			let text_len = test_text.len();
			if let Some(regex) = regex {
				let test_captures = regex.captures_iter(test_text)
				.flat_map(|found| {
					found.iter_pos().enumerate().filter_map(|(group_index, group)| {
						group.map(|(start, end)| CaptureInfo::from((group_index, start, end)))
					}).collect::<Vec<CaptureInfo>>()
				}).collect::<Vec<CaptureInfo>>();
				let test_captures = CaptureInfoFillIter::new(
					test_captures, text_len).collect();
				// print!("\n\n\n\n\n\n\n==========\n\n");
				// dbg!(&test_captures);
				Some(test_captures)
			} else {
				None
			}
		};
		let update_regex = |regex_pattern: &str, test_text: &str| {
			let (regex, regex_error) = match regex_pattern.len() {
				0 => (None, String::default()),
				_ => {
					match Regex::new(&regex_pattern) {
						Ok(r) => (Some(r), String::default()),
						Err(e) => (None, e.to_string())
					}
				},
			};
			let test_captures = update_text(regex.as_ref(), test_text);
			(regex, test_captures, regex_error)
		};

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.horizontal(|ui| {
	ui.label("Regex:");
	let regex_is_valid = self.regex.is_some();
	let mut layouter = |ui: &egui::Ui, text: &str, wrap_width| {
		let mut layout_job = LayoutJob::default();
		layout_job.wrap.max_width = wrap_width;
		layout_job.append(text, 0., TextFormat {
			color: if regex_is_valid {
				ui.style().visuals.text_color()
			} else {
				Color32::RED
			},
			font_id: FontId::monospace(14.),
			..Default::default()
		});
		ui.fonts().layout_job(layout_job)
	};
	let text_edit = egui::TextEdit::singleline(&mut self.regex_str)
		.layouter(&mut layouter).code_editor();
	if ui.add(text_edit).changed() {
		// If you have a nested closure that borrows self with a capture, that
		// borrow lasts for the entire outer scope. Why? I think it's because
		// closures are variables.
		// I worked around the borrow check by taking advantage of passing
		// values/references to and returning values from the closure.
		(self.regex, self.test_captures, self.regex_error) = update_regex(&self.regex_str, &self.test_text);
	}
			});
			if self.regex_error.len() > 0 {
				ui.colored_label(Color32::RED, &self.regex_error);
			}
			ui.horizontal(|ui| {
				ui.checkbox(&mut self.test_text_monospace, "Monospace");
				ui.add(egui::Slider::new(&mut self.test_text_size, 10.0..=36.0)
				.step_by(1.0).suffix("px").text("Size"));
			});
			ui.label("Test text:");
			egui::ScrollArea::vertical().show(ui, |ui| {
				let mut layouter = |ui: &egui::Ui, text: &str, wrap_width| {
					let mut layout_job = LayoutJob::default();
					layout_job.wrap.max_width = wrap_width;
					let coloured_format = |hue| {
						TextFormat {
							color: match hue {
								v if v > 0. && v <= HUE_MAX => hue_to_rgb(hue),
								_ => ui.style().visuals.text_color(),
							},
							font_id: match self.test_text_monospace {
								true => FontId::monospace(self.test_text_size),
								false => FontId::proportional(self.test_text_size)
							},
							..Default::default()
						}
					};
					if let Some(caps) = self.test_captures.as_ref() {
						caps.iter().for_each(|cap| {
							let range = cap.range.0..cap.range.1;
							let hue = match cap.group {
								Some(g) => HUE_MIN * ((g + 1) as f32).rem_euclid(HUE_MAX),
								None => 0.,
							};
							if let Some(tslice) = text.get(range) {
								layout_job.append(tslice, 0.,
									coloured_format(hue));
							}
						});
					} else {
						layout_job.append(text, 0., coloured_format(0.));
					}
					ui.fonts().layout_job(layout_job)
				};
				let text_edit = egui::TextEdit::multiline(&mut self.test_text)
					.layouter(&mut layouter);
				if ui.add_sized(ui.available_size(), text_edit).changed() {
					self.test_captures = update_text(self.regex.as_ref(), &self.test_text);
				}
			});
		});
	}
}
