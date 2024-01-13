use crate::category::{CategoryName, CategoryManager};
use crate::csvadapter::*;
use crate::entry::{Cost, Entry};
use crate::organize::*;
use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap;
use std::path::PathBuf;

type Comparator = Box<dyn Fn(&Entry, &Entry) -> std::cmp::Ordering>;
type CostMap = BTreeMap<CategoryName, BTreeMap<NaiveDate, f32>>;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct DataManager {
    // entries are loaded from/written to this file on native
    pub active_file: Option<PathBuf>,

    pub sort_by: SortBy,

    #[serde(skip)]
    // We don't serialize entries because the underlying data could have changed, so we reload it
    pub entries: Vec<Entry>,

    #[serde(skip)]
    /// Was data loaded recently? This is meant to share state with the rest of the app.
    /// plotter will reset it once it's done a reset
    pub plot_reset_next_frame: bool,
}

impl Default for DataManager {
    fn default() -> Self {
        Self {
            entries: vec![],
            sort_by: SortBy::Date,
            active_file: None,
            plot_reset_next_frame: false,
        }
    }
}

// Basic Accessors
impl DataManager {
    // Returns either a normal or reverse iterator depending on the 'reverse' parameter
    // This allows the backend to not have to worry about whether the entry list is reversed
    // iter() and iter.rev() are different types, so they're wrapped in a Box so that the return
    // type size is known at compile time
    // overall this makes for a clean API where you can get the normal or reversed list with 1
    // parameter instead of 2 functions or keeping extra state (and doing extra sorting with certain
    // operations) in the backend.
    // this is meant to be used like:
    // for entry: &Entry in backend.get_entries_iter(reverse)
    pub fn get_entries_iter(&self, reverse: bool) -> Box<dyn Iterator<Item = &Entry> + '_> {
        if reverse {
            Box::new(self.entries.iter().rev())
        } else {
            Box::new(self.entries.iter())
        }
    }
}

impl DataManager {
    pub fn read_entries_from_csv(&mut self, file_path: PathBuf) {
        let result = read_entries_from_file(&file_path);

        match result {
            Ok(entries) => {
                self.entries = entries;
                // set some flag so we know to reset the plot
                self.plot_reset_next_frame = true;
            }
            Err(e) => error!("Error reading entries from file \"{:?}\": {}", file_path, e),
        }

        // TODO: clean this up?
        if self.active_file != Some(file_path.clone()) {
            self.active_file = Some(file_path);
            // self.serialize_backend();
        }
    }

    /// Write entries to CSV
    ///
    /// If `file_path` is None, attempts to write to the active file. If no active file is set, does nothing
    /// If `file_path` is specified, will update the active file if they don't match
    pub fn write_entries_to_csv(&mut self, file_path: Option<PathBuf>) {
        let file_path = match file_path {
            Some(path) => path,
            None => {
                match &self.active_file {
                    Some(path) => path.clone(),
                    None => {
                        debug!("write entries to CSV with unspecified path & no active file - skipping");
                        return;
                    }
                }
            }
        };

        if let Err(e) = write_entries_to_csv(&self.entries, &file_path) {
            error!("Error writing entries to CSV: {}", e);
        }

        if self.active_file.as_ref() != Some(&file_path) {
            self.active_file = Some(file_path);
            // self.serialize_backend();
        }
    }

    // some data changed in entries (as a result of UI interaction)
    // for now, this is just called on add/delete
    fn data_changed(&mut self) {
        debug!("Data changed, calling write_entries_to_csv");
        self.write_entries_to_csv(None);
    }

    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);

        // what were we sorted by? ensure that we're still sorted
        self.sort_entries(self.sort_by);

        self.data_changed();
    }

    pub fn remove_entry_pos(&mut self, index: usize) {
        self.entries.remove(index);

        self.data_changed();
    }

    pub fn sort_entries(&mut self, sort_by: SortBy) {
        let comparator: Comparator = match sort_by {
            SortBy::Cost => Box::new(|a, b| {
                Into::<f32>::into(a.cost)
                    .partial_cmp(&b.cost.into())
                    .unwrap()
            }),
            SortBy::Date => Box::new(|a, b| {
                let date_a = &a.date;
                let date_b = &b.date;

                if date_a.year() != date_b.year() {
                    date_a.year().cmp(&date_b.year())
                } else if date_a.month() != date_b.month() {
                    date_a.month().cmp(&date_b.month())
                } else {
                    date_a.day().cmp(&date_b.day())
                }
            }),
        };
        self.entries.sort_by(|a, b| comparator(a, b));

        // only serialize if it's changed
        if self.sort_by != sort_by {
            self.sort_by = sort_by;
            // self.serialize_backend();
        }
    }

    /// Builds an ordered mapping for each date to the total spent on that date.
    /// order the category map so it's always sorted the same. If you use a hashmap it's in a different order for every
    /// frame, which makes them get a different color
    pub fn cost_map(&self, group_by: GroupBy, categories: Vec<CategoryName>) -> CostMap {
        if self.entries.is_empty() {
            return BTreeMap::new();
        }
        // build the cost map with zerod entries accordingly
        let mut map = self.zero_cost_map(categories, group_by);

        // now track a sum for each date
        for entry in self.get_entries_iter(false) {
            if !categories.contains(&entry.category) {
                continue; // skip anything that wasn't asked for
            }
            // scale the date based on the grouping. if grouping by month, all entries are counted for the first day of the month
            // if grouping by year, all entries are counted for the first day of the year. This has to match how zero_cost_map
            // builds the map.
            let scaled_date = match group_by {
                GroupBy::Day => entry.date,
                GroupBy::Month => {
                    NaiveDate::from_ymd_opt(entry.date.year(), entry.date.month(), 1).unwrap()
                }
                GroupBy::Year => NaiveDate::from_ymd_opt(entry.date.year(), 1, 1).unwrap(),
            };
            let inner_map = map.entry(entry.category).or_default();
            let sum = inner_map.entry(scaled_date).or_insert(0.0);
            let cost: f32 = entry.cost.into();
            *sum += cost;
        }
        map
    }

    // get the total spent in a given category given a category, month & year
    // NOTE: the 'day' component of 'date' is ignored, it's just simpler to have 1 parameter
    pub fn monthly_cost(&self, _category: CategoryName, _date: NaiveDate) -> f32 {
        0.0 // TODO: implement me
    }

    // TODO: think of a better design for storing entries so I don't have to do this?
    // could cache first and last to make it easier.
    // this is necessary because the UI can sort entries by cost and we sort the backend vect accordingly
    // but we need the first and last dates to generate zero_cost_map()
    fn entries_date_extremes(&self) -> (Option<&Entry>, Option<&Entry>) {
        let mut earliest: Option<&Entry> = None;
        let mut latest: Option<&Entry> = None;

        for entry in &self.entries {
            match earliest {
                None => earliest = Some(entry),
                Some(e) if entry.date < e.date => earliest = Some(entry),
                _ => {}
            }
            match latest {
                None => latest = Some(entry),
                Some(e) if entry.date > e.date => latest = Some(entry),
                _ => {}
            }
        }

        (earliest, latest)
    }

    // return a map filled with zeros for every day between the first and last entries, inclusive
    // takes into account the grouping - entries look like:
    // 1/1/xxxx, 2/1/xxxx, 3/1/xxxx, etc for GroupBy::Month
    // 1/1/xxxx, 1/1/xxxx + 1, 1/1/xxxx + 2, for GroupBy::Year
    // Assumes the entry map has something in it.
    fn zero_cost_map(&self, categories: Vec<CategoryName>, group_by: GroupBy) -> CostMap {
        let (first, last) = self.entries_date_extremes();
        let first_days = first.unwrap().date.num_days_from_ce();
        let last_days = last.unwrap().date.num_days_from_ce();
        categories.iter()
            .map(|category| {
                let dates = (first_days..=last_days)
                    .filter_map(|days| {
                        let date = NaiveDate::from_num_days_from_ce_opt(days)?;
                        match group_by {
                            GroupBy::Day => Some(date),
                            GroupBy::Month if date.day() == 1 => Some(date),
                            GroupBy::Year if date.month() == 1 && date.day() == 1 => Some(date),
                            _ => None,
                        }
                    })
                    .map(|date| (date, 0.0))
                    .collect::<BTreeMap<NaiveDate, f32>>();
                (category.clone(), dates)
            })
            .collect()
    }
}

mod tests {
    use super::*;

    const CATEGORIES: [&str; 5] = ["test1", "test2", "test3", "test4", "test5"];

    fn random_category() -> CategoryName {
        use rand::Rng;
        use std::str::FromStr;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..CATEGORIES.len());
        let name = CATEGORIES[index];
        CategoryName::from_str(name).unwrap()
    }

    /// Modify the backend in place. give it a random list of (sorted) entries of a particular size
    fn _fill_entries(size: usize, backend: &mut DataManager) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut entries = Vec::with_capacity(size);

        for i in 0..size {
            let year = rng.gen_range(2000..=2023);
            let month = rng.gen_range(1..=12);
            let day = match month {
                2 => rng.gen_range(1..=28), // Simplification: not handling leap years.
                4 | 6 | 9 | 11 => rng.gen_range(1..=30),
                _ => rng.gen_range(1..=31),
            };

            let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();

            entries.push(Entry {
                name: format!("entry{}", i),
                cost: Cost::try_from(rng.gen_range(1.0..=500.0)).unwrap(),
                date,
                category: random_category(),
            });
        }

        backend.entries = entries;
        backend.sort_entries(SortBy::Date); // this shouldn't cause serialization, but if it does we need to get to
    }

    /// TODO: make this work for other groupings
    #[test]
    fn test_cost_map() {
        use chrono::Duration;
        use std::str::FromStr;
        let mut backend = DataManager::default();
        tests::_fill_entries(1_000_000, &mut backend);

        let (first, last) = backend.entries_date_extremes();
        let first_days = first.unwrap().date.num_days_from_ce();
        let last_days = last.unwrap().date.num_days_from_ce();

        let cat_names = CATEGORIES;
        let mut categories = Vec::new();
        for cat in cat_names.iter() {
            categories.push(CategoryName::from_str(cat).unwrap());
        }
        let map = backend.cost_map(GroupBy::Day, categories.clone().to_vec());

        // map should have a key for every category
        assert!(categories.iter().all(|category| map.contains_key(category)));

        // each category should correspond to a key for every day in the date range
        let last_day = NaiveDate::from_num_days_from_ce_opt(last_days).unwrap();
        for category in categories.iter() {
            let dates_map = map.get(category).expect("Category not found in map");
            let mut date = NaiveDate::from_num_days_from_ce_opt(first_days).unwrap();
            while date <= last_day {
                assert!(
                    dates_map.contains_key(&date),
                    "Date {:?} not found for category {:?}",
                    date,
                    category
                );
                date += Duration::days(1);
            }
        }
    }
    // TODO: mock the serializer to allow testing without any actual file interaction
}
