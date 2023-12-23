use super::super::App;
use egui::{Color32, RichText, Ui};

pub struct MainPage {}

/// The top level page for the Rudget app. Draws everything except the MenuBar
impl MainPage {
    pub fn add(app: &mut App, ui: &mut Ui, frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.heading("PennyPilot");
            global_dark_light_mode_switch(ui);

            #[cfg(not(target_arch = "wasm32"))] // no screenshots on web
            if ui.button("Take Screenshot").clicked() {
                debug!("Requesting screenshot");
                frame.request_screenshot(); // it's gathered and written out during post_rendering()
            }
        });

        Self::links(ui);

        ui.separator();

        // no concept of an active file on wasm
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::active_file(ui, app);
            ui.separator();
        }

        // show the 'add entry' ui. Check spending limits if something was added
        if let Some(entry) = app.add_entry_view.ui(ui, &mut app.data_mgr) {
            debug!("Entry added! Checking spending limits.");
            app.spending_limits.check_limit(&entry, &app.data_mgr);
        }

        // show the graph ui
        app.graph.ui(ui, &mut app.data_mgr);
    }

    fn links(ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.hyperlink_to("Source Code", "https://github.com/benliepert/PennyPilot");
            ui.label(". Powered by ");
            ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            ui.label(" and ");
            ui.hyperlink_to(
                "eframe",
                "https://github.com/emilk/egui/tree/master/crates/eframe",
            );
            ui.label(".");
            egui::warn_if_debug_build(ui);
        });
    }

    fn active_file(ui: &mut Ui, app: &mut App) {
        ui.horizontal(|ui| {
            ui.label("Active file: ");
            let (label, color) = if let Some(file) = &app.data_mgr.active_file {
                (format!("{:?}", file), Color32::DARK_GREEN)
            } else {
                (
                    "No active file set. Use File -> Import to add one".to_string(),
                    Color32::YELLOW,
                )
            };

            ui.label(RichText::new(label).color(color));
        });
    }
}

/// Copied this code from the egui demo
fn global_dark_light_mode_switch(ui: &mut egui::Ui) {
    let style: egui::Style = (*ui.ctx().style()).clone();
    let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
    if let Some(visuals) = new_visuals {
        ui.ctx().set_visuals(visuals);
    }
}
