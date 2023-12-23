use crate::backend::DataManager;
use crate::category::Category;
use crate::entry::{Cost, Entry};
use chrono::NaiveDate;
use egui::Ui;
use strum::IntoEnumIterator;

pub struct AddEntry {
    date: NaiveDate,
    cost: f32,
    name: String,
    category: Category,
    // are we allowed to add an entry? (all fields must be filled out)
}

impl Default for AddEntry {
    fn default() -> Self {
        Self {
            date: chrono::offset::Utc::now().date_naive(),
            cost: 0.0,
            name: "".to_string(),
            category: Category::Misc,
        }
    }
}

impl AddEntry {
    // returns None if nothing was added in the UI, and Some(Entry) if an entry was.
    pub fn ui(&mut self, ui: &mut Ui, backend: &mut DataManager) -> Option<Entry> {
        let mut entry_opt = None;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui_extras::DatePickerButton::new(&mut self.date));
                ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("Enter purchase name"));
                ui.add(
                    // for price:
                    egui::DragValue::new(&mut self.cost)
                        .speed(2.5)
                        .clamp_range(0.0..=10_000.0)
                        .prefix("$"),
                );
                egui::ComboBox::from_id_source("category")
                    .selected_text(format!("{}", self.category))
                    // TODO: make width dynamic based on the widest category title
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for category in Category::iter() {
                            if category != Category::All {
                                ui.selectable_value(
                                    &mut self.category,
                                    category,
                                    category.to_string(),
                                );
                            }
                        }
                    });

                // don't require an active file to start adding entries - you just need to remember to export!
                let add_enabled = !self.name.trim().is_empty() && self.cost != 0.0;
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
        Entry {
            name: self.name.clone(),
            cost: Cost::try_from(self.cost).unwrap(),
            date: self.date,
            category: self.category,
        }
    }
}
