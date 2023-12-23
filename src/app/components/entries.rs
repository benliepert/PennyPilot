use crate::backend::DataManager;
use crate::organize::*;
use egui::Ui;
use strum::IntoEnumIterator;

pub struct Entries {
    pub sort_by: SortBy,
    pub sort_order: SortOrder,

    // allow deleting entries from the view. If true, a "delete" button will be clickable next to each entry
    pub allow_deletion: bool,
}

impl Default for Entries {
    fn default() -> Self {
        debug!("default called for entries");
        Self {
            sort_by: SortBy::Date,
            sort_order: SortOrder::Increasing,
            allow_deletion: false,
        }
    }
}

/// A vertical scroll area for inspecting, sorting, and deleting entry objects
impl Entries {
    pub fn ui(&mut self, ui: &mut Ui, data_mgr: &mut DataManager) {
        self.controls(ui, data_mgr);
        self.scroll_area(ui, data_mgr);
    }

    fn controls(&mut self, ui: &mut Ui, data_mgr: &mut DataManager) {
        ui.horizontal(|ui| {
            ui.label("Sort By:"); // I like the label on the left
            egui::ComboBox::from_id_source("sort-by")
                .selected_text(format!("{}", self.sort_by))
                .show_ui(ui, |ui| {
                    let cur_sort = self.sort_by;
                    for sort in SortBy::iter() {
                        ui.selectable_value(&mut self.sort_by, sort, sort.to_string());
                    }
                    if cur_sort != self.sort_by {
                        data_mgr.sort_entries(self.sort_by);
                    }
                });
            egui::ComboBox::from_id_source("sort-order")
                .selected_text(format!("{}", self.sort_order))
                .show_ui(ui, |ui| {
                    for order in SortOrder::iter() {
                        ui.selectable_value(&mut self.sort_order, order, order.to_string());
                    }
                });
            if ui
                .add(egui::RadioButton::new(
                    self.allow_deletion,
                    "Allow Deletion",
                ))
                .clicked()
            {
                self.allow_deletion = !self.allow_deletion;
            }
        });
    }

    fn scroll_area(&mut self, ui: &mut Ui, data_mgr: &mut DataManager) {
        egui::ScrollArea::vertical()
            // .max_width(400.0)
            .show(ui, |ui| {
                if data_mgr.entries.is_empty() {
                    ui.label("(No Entries)");
                }
                let mut to_delete = Vec::new();
                let reversed = self.sort_order == SortOrder::Decreasing;
                for (index, entry) in data_mgr.get_entries_iter(reversed).enumerate() {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{}: {}, {} (${:.2})",
                                &entry.date.to_string(),
                                &entry.name,
                                &entry.category,
                                Into::<f32>::into(entry.cost),
                            ));
                            ui.add_enabled_ui(self.allow_deletion, |ui| {
                                if ui.button("Delete").clicked() {
                                    // we can't delete the entry while we're iterating the entries
                                    to_delete.push(index);
                                }
                            });
                        });
                        ui.separator();
                    });
                }

                if to_delete.len() > 1 {
                    debug!("More than 1 entry to delete this frame. That's weird");
                }

                // now that we're done iterating, it's safe to delete the entries. It should only be 1 unless
                // the user is very fast or framerate very slow. Iterate backwards because each deletion would
                // shift other indices (but again, there should only be 1)
                for index in to_delete.iter().rev() {
                    // if we're viewing the list reversed, we need to translate the index so that we remove the
                    // right thing from the backend.
                    let mut idx = *index;
                    if self.sort_order == SortOrder::Decreasing {
                        // it shouldn't be possible to delete an entry if there's nothing in the list
                        // if that happens, just let it panic on the invalid index
                        idx = data_mgr.entries.len() - 1 - idx;
                    }
                    data_mgr.remove_entry_pos(idx);
                }
            });
    }
}
