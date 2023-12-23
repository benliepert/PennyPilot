use crate::app::App;

#[cfg(not(target_arch = "wasm32"))]
use crate::app::SCREENSHOT_PATH;

use super::components::{MainPage, MenuBar};

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::add(self, ui, frame);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            MainPage::add(self, ui, frame);
            self.show_windows(ui);
        });
    }

    // Screenshot not supported on wasm. See: https://docs.rs/eframe/latest/eframe/struct.Frame.html#method.screenshot
    #[cfg(not(target_arch = "wasm32"))]
    fn post_rendering(&mut self, _window_size_px: [u32; 2], frame: &eframe::Frame) {
        if let Some(screenshot) = frame.screenshot() {
            let pixels_per_point = frame.info().native_pixels_per_point;
            // taking a shot of the full screen
            let x_size = frame.info().window_info.size.x;
            let y_size = frame.info().window_info.size.y;
            let region = egui::Rect::from_two_pos(
                egui::Pos2::ZERO,
                egui::Pos2 {
                    x: x_size,
                    y: y_size,
                },
            );
            let region = screenshot.region(&region, pixels_per_point);
            let filename = format!(
                "{}/screenshot_{}_{}.png",
                SCREENSHOT_PATH,
                chrono::Utc::now().timestamp(),
                chrono::Utc::now().timestamp_subsec_millis()
            );

            // Ensure the directories exist
            if let Err(e) = std::fs::create_dir_all(format!("{}/", SCREENSHOT_PATH)) {
                error!("Failed to create directory: {}", e);
            }

            image::save_buffer(
                &filename,
                region.as_raw(),
                region.width() as u32,
                region.height() as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();

            debug!("Saved screenshot to: {}", filename);
        }
    }
}
