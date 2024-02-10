use chrono::{Datelike, NaiveDate};
use core::fmt;
use egui::Ui;
use std::collections::BTreeMap;
use std::str::FromStr;

use crate::datamanager::DataManager;
use crate::entry::Entry;

#[derive(
    Ord, PartialOrd, PartialEq, Eq, serde::Deserialize, serde::Serialize, Clone, Debug, Hash,
)]
pub struct CategoryName(String);

impl std::error::Error for CategoryError {}

impl FromStr for CategoryName {
    type Err = CategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.trim().is_empty() && s.chars().all(|c| c.is_alphabetic() || c.is_whitespace()) {
            Ok(CategoryName(s.to_lowercase()))
        } else {
            Err(CategoryError::Invalid(s.to_string()))
        }
    }
}

impl fmt::Display for CategoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CategoryInfo {
    pub limit: Option<f32>,
    /// is the category displayed in the UI?
    // This introduces some coupling, but I think it's fine since it keeps it simple
    pub displayed: bool,
}

impl Default for CategoryInfo {
    fn default() -> Self {
        CategoryInfo {
            limit: None,
            displayed: true,
        }
    }
}

/// Stores user defined categories. Case insensitive. Also manages spending limits.
#[derive(Default, Clone)]
pub struct CategoryManager {
    pub categories: BTreeMap<CategoryName, CategoryInfo>,

    /// whether to warn the user when they exceed a category's spending limit
    // TODO: move this somewhere else?
    spending_warnings_enabled: bool,

    new_category: String,
}

#[derive(PartialEq, Eq, Debug)]
pub enum CategoryError {
    /// The category already exists
    Duplicate(String),
    /// The category name is invalid
    Invalid(String),
}

impl std::fmt::Display for CategoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CategoryError::Duplicate(cat) => write!(f, "Category already exists: {cat}"),
            CategoryError::Invalid(cat) => write!(f, "Invalid category name: {cat}"),
        }
    }
}

// TODO: if adding a remove method, need to be careful because limit checks will panic if we add
// an entry with a category that doesn't exist. Not sure if this is a real issue, as adding the
// entry implies a new category in this scenario, but I haven't decided how to implement category
// creation yet, and it won't necessarily be when the entry is added.
impl CategoryManager {
    pub fn add(&mut self, category: String) -> Result<(), CategoryError> {
        let new_category = CategoryName::from_str(&category.to_lowercase())?;

        if self.categories.contains_key(&new_category) {
            debug!("Category already exists: {}", new_category);
            return Err(CategoryError::Duplicate(new_category.to_string()));
        }

        debug!("Adding category: {}", new_category);
        self.categories
            .insert(new_category, CategoryInfo::default());
        Ok(())
    }

    /// Append a list of categories to the manager. Duplicate categories are ignored.
    fn append_categories<I>(&mut self, categories: I) -> Result<(), CategoryError>
    where
        I: IntoIterator<Item = CategoryName>,
    {
        for category in categories {
            // converting to string is wasteful here, since we already have it as a CategoryName
            match self.add(category.to_string()) {
                Ok(_) => (),
                Err(CategoryError::Duplicate(s)) => {
                    debug!("Duplicate category '{s}' ignored.");
                }
                // all other errors are legit
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn selected_categories(&self) -> Vec<CategoryName> {
        self.categories
            .iter()
            .filter(|(_, &info)| info.displayed)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Returns a list of all categories, sorted alphabetically
    // returns a slice to avoid copying the vector
    pub fn categories(&self) -> Vec<&CategoryName> {
        self.categories.keys().collect()
    }

    // TODO: this is functionality I've generally reserved for the "components" subdir...
    // should move it there/refactor
    pub fn limits_ui(&mut self, ui: &mut Ui, _backend: &DataManager) {
        let today = Self::current_date();
        let hover_text = format!("When checked, Rudget will warn you if you exceed a category-wise spending limit when adding an entry. Note that warnings only apply to entries added in the current month ({}/{}).", today.month(), today.year());
        ui.checkbox(
            &mut self.spending_warnings_enabled,
            "Enable Spending Warnings",
        )
        .on_hover_text(hover_text);
        egui::Grid::new("spending-limits-grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (category, info) in self.categories.iter_mut() {
                    ui.horizontal(|ui| {
                        let limit = &mut info.limit;
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

    pub fn editor_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_category);
            if ui.button("Add").clicked() {
                if let Err(e) = self.add(self.new_category.clone()) {
                    debug!("Failed to add category: {}", e);
                }
                self.new_category.clear();
            }
        });
        ui.separator();
        let mut to_delete = Vec::new();
        for category in self.categories() {
            ui.horizontal(|ui| {
                ui.label(category.to_string());
                if ui.button("Delete").clicked() {
                    to_delete.push(category.clone());
                }
            });
        }
        for category in to_delete {
            debug!("Deleting category: {}", category);
            self.categories.remove(&category);
        }
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
        if !self.spending_warnings_enabled {
            debug!("Warnings are disabled. Skipping spending limits check");
            return;
        }

        let cat_name = entry.category.clone();
        if let Some(limit) = self.categories[&cat_name].limit {
            // get the sum of all items in the category for the month from the backend
            // what about retroactively adding entries? should we still be warned for those?
            // should check the relevant date range and only warn for items added in the current month?

            // for now, only going to make this work for entries that are added to the current month.
            // anything else will be considered retroactive, and spending limits won't generate a warning.
            // limits can additionally be graphed though so you can see when they're exceeded
            let today: NaiveDate = Self::current_date();
            if today.month() == entry.date.month() && today.year() == entry.date.year() {
                // get the sum for entry.category for this date
                let cost = backend.monthly_cost(&entry.category, &entry.date);

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

mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut manager = CategoryManager::default();

        assert_eq!(
            manager.add("".to_string()),
            Err(CategoryError::Invalid("".to_string()))
        );
        assert_eq!(
            manager.add(" ".to_string()),
            Err(CategoryError::Invalid(" ".to_string()))
        );

        assert_eq!(
            manager.add("_abcde!.eg/".to_string()),
            Err(CategoryError::Invalid("_abcde!.eg/".to_string()))
        );

        assert_eq!(manager.add("arsts".to_string()), Ok(()));
        assert_eq!(
            manager.add("ARSTS".to_string()),
            Err(CategoryError::Duplicate("arsts".to_string()))
        );
        assert_eq!(
            manager.add("ArsTS".to_string()),
            Err(CategoryError::Duplicate("arsts".to_string()))
        );

        assert_eq!(manager.add("validcategory".to_string()), Ok(()));
        assert_eq!(manager.add(" arsts ".to_string()), Ok(()));
    }
}
