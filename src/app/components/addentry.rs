use crate::category::{CategoryManager, CategoryName};
use crate::datamanager::DataManager;
use crate::entry::{Cost, Entry};
use chrono::NaiveDate;
use egui::Ui;

const MAX_ENTRY_PRICE: u32 = 10_000;

pub struct AddEntry {
    date: NaiveDate,
    cost: f32,
    name: String,
    // TODO: figure out what this should be
    // String? CategoryName?
    category: String,
    // are we allowed to add an entry? (all fields must be filled out)
}

impl Default for AddEntry {
    fn default() -> Self {
        Self {
            date: chrono::offset::Utc::now().date_naive(),
            cost: 0.0,
            name: "".to_string(),
            category: "".to_string(),
        }
    }
}

impl AddEntry {
    // returns None if nothing was added in the UI, and Some(Entry) if an entry was.
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        backend: &mut DataManager,
        cat_mgr: &CategoryManager,
    ) -> Option<Entry> {
        let mut entry_opt = None;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui_extras::DatePickerButton::new(&mut self.date));
                ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("Enter purchase name"));
                ui.add(
                    // for price:
                    egui::DragValue::new(&mut self.cost)
                        .speed(2.5)
                        .clamp_range(0.0..=MAX_ENTRY_PRICE as f32)
                        .prefix("$"),
                );
                egui::ComboBox::from_id_source("category")
                    .selected_text(self.category.to_string())
                    // TODO: make width dynamic based on the widest category title
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for category in cat_mgr.categories() {
                            ui.selectable_value(
                                &mut self.category,
                                category.to_string(),
                                category.to_string(),
                            );
                        }
                    });

                // don't require an active file to start adding entries - you just need to remember to export!
                let add_enabled =
                    !self.name.trim().is_empty() && self.cost != 0.0 && !self.category.is_empty();
                if ui
                    .add_enabled(add_enabled, egui::Button::new("Add"))
                    .clicked()
                {
                    let entry = self.build_entry();
                    entry_opt = Some(entry.clone());
                    backend.add_entry(entry);
                }
            });
        });
        entry_opt
    }

    // build an Entry based on what's currently filled in in the UI
    fn build_entry(&self) -> Entry {
        use std::str::FromStr;
        Entry {
            name: self.name.clone(),
            cost: Cost::try_from(self.cost).unwrap(),
            date: self.date,
            category: CategoryName::from_str(&self.category).expect(
                "selectable values should all be valid because they come from the category mgr",
            ),
        }
    }
}
