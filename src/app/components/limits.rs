use crate::backend::DataManager;
use crate::category::Category;
use crate::entry::Entry;
use chrono::Datelike;
use chrono::NaiveDate;
use egui::Ui;
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub struct Limits {
    // each category has an optional spending limit associated with it
    // this will be used to warn the user when they're spending too much :)
    // for now, this is ONLY by month.
    limits: HashMap<Category, Option<f32>>,

    warnings_enabled: bool,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            limits: Category::iter().map(|category| (category, None)).collect(),
            warnings_enabled: true,
        }
    }
}

// display spending limits
impl Limits {
    pub fn ui(&mut self, ui: &mut Ui, _backend: &DataManager) {
        let today = Limits::current_date();
        let hover_text = format!("When checked, Rudget will warn you if you exceed a category-wise spending limit when adding an entry. Note that warnings only apply to entries added in the current month ({}/{}).", today.month(), today.year());
        ui.checkbox(&mut self.warnings_enabled, "Enable Spending Warnings")
            .on_hover_text(hover_text);
        egui::Grid::new("spending-limits-grid")
            .num_columns(2)
            // .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for (category, limit) in self.limits.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.label(category.to_string());
                        let mut display_value = limit.unwrap_or(0.0);
                        ui.add(
                            egui::DragValue::new(&mut display_value)
                                .speed(10.0)
                                .clamp_range(0.0..=1_000_000.0)
                                .prefix("$"),
                        );

                        *limit = if display_value == 0.0 {
                            None
                        } else {
                            Some(display_value)
                        };
                    });
                    ui.end_row();
                }
            });
    }

    fn current_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(
            chrono::Local::now().year(),
            chrono::Local::now().month(),
            chrono::Local::now().day(),
        )
        .unwrap()
    }

    // check whether a limit was exceeded with the addition of 'entry'
    pub fn check_limit(&self, entry: &Entry, backend: &DataManager) {
        if !self.warnings_enabled {
            debug!("Warnings are disabled. Skipping spending limits check");
            return;
        }
        if let Some(limit) = self.limits[&entry.category] {
            // get the sum of all items in the category for the month from the backend
            // what about retroactively adding entries? should we still be warned for those?
            // should check the relevant date range and only warn for items added in the current month?

            // for now, only going to make this work for entries that are added to the current month.
            // anything else will be considered retroactive, and spending limits won't generate a warning.
            // limits can additionally be graphed though so you can see when they're exceeded
            let today: NaiveDate = Limits::current_date();
            if today.month() == entry.date.month() && today.year() == entry.date.year() {
                // get the sum for entry.category for this date
                let cost = backend.monthly_cost(entry.category, entry.date);

                if cost >= limit {
                    // limit has been met/exceeded!
                    // warn the user
                    warn!(
                        "Limit for category: {} (${}) has been exceeded!",
                        entry.category, limit
                    );
                } else {
                    debug!(
                        "Limit for category: {} (${}) has NOT been exceeded. Total cost is {cost}",
                        entry.category, limit
                    );
                }
            } else {
                debug!("Entry's date doesn't match the current month. Skipping limit check");
            }
        } else {
            debug!("No limit set for category: {}", entry.category);
        }
    }
}
