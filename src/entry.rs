use crate::category::Category;

use chrono::NaiveDate;
use csv::StringRecord;
use std::error::Error;
use std::str::FromStr;

#[derive(Copy, serde::Deserialize, serde::Serialize, PartialEq, Clone, Debug)]
pub struct Cost(f32);

// a cost must be a positive number, and can only be created with this function
impl TryFrom<f32> for Cost {
    type Error = f32;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value >= 0.0 {
            Ok(Cost(value))
        } else {
            Err(value)
        }
    }
}

impl From<Cost> for f32 {
    fn from(cost: Cost) -> Self {
        cost.0
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Entry {
    pub name: String,
    pub cost: Cost,
    pub date: NaiveDate,
    pub category: Category,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.date == other.date && self.category == other.category
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entry(name: {}, cost: {:?}, date: {})",
            self.name, self.cost, self.date
        )
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            cost: Cost::try_from(0.0).unwrap(),
            date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            category: Category::Misc,
        }
    }
}

impl TryFrom<StringRecord> for Entry {
    type Error = Box<dyn Error>;

    fn try_from(record: StringRecord) -> Result<Self, Self::Error> {
        if record.len() != 4 {
            return Err("Record must have exactly 4 fields".into());
        }

        let name = record[0].to_string();
        let date = NaiveDate::parse_from_str(&record[1], "%Y-%m-%d")?;
        let cost = record[2].parse::<f32>()?;
        let category = Category::from_str(&record[3])?;

        Ok(Entry {
            name,
            cost: Cost::try_from(cost).map_err(|_| "Invalid cost")?,
            date,
            category,
        })
    }
}

impl Entry {
    /// Todo: convert this to produce a StringRecord and make this use the csv crate interface on the other side too?
    pub fn to_csv_string(&self) -> String {
        format!(
            "{},{},{:?},{}",
            self.name,
            self.date,
            Into::<f32>::into(self.cost),
            self.category
        )
    }
}
