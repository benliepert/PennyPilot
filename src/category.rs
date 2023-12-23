use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Types of purchases
#[derive(
    serde::Deserialize,
    serde::Serialize,
    EnumIter,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Debug,
)]
pub enum Category {
    // if you add an enum, make sure you update get_random
    // NOTE: these are in reverse sorted order so that they visually match up with the legend on the plot
    // since it's generated with EnumIter::iter(). The cost map is sorted as well, but the first category
    // to be displayed goes at the bottom. So for these to match up the legend should appear reversed
    Travel,
    Subscriptions,
    Rent,
    OtherFood,
    Misc,
    Groceries,
    GiftsForSelf,
    GiftsForOthers,
    Clothes,
    Car,
    All, // special category for querying the costmap from the backend. Ignored when adding entries in the UI
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Category::*;
        match *self {
            All => write!(f, "All"),
            Groceries => write!(f, "Groceries"),
            OtherFood => write!(f, "Other Food"),
            GiftsForOthers => write!(f, "Gifts for Others"),
            GiftsForSelf => write!(f, "Gifts for Self"),
            Clothes => write!(f, "Clothes"),
            Car => write!(f, "Car"),
            Rent => write!(f, "Rent"),
            Travel => write!(f, "Travel"),
            Subscriptions => write!(f, "Subscriptions"),
            Misc => write!(f, "Misc"),
        }
    }
}

impl FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Category::*;
        match s {
            "Groceries" => Ok(Groceries),
            "Other Food" => Ok(OtherFood),
            "Gifts for Others" => Ok(GiftsForOthers),
            "Gifts for Self" => Ok(GiftsForSelf),
            "Clothes" => Ok(Clothes),
            "Car" => Ok(Car),
            "Rent" => Ok(Rent),
            "Travel" => Ok(Travel),
            "Misc" => Ok(Misc),
            "Subscriptions" => Ok(Subscriptions),
            "All" => Ok(All),
            _ => Err(format!("Invalid category: {}", s)),
        }
    }
}

impl Category {
    pub fn _get_random() -> Self {
        use rand::Rng;
        use Category::*;
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..10) {
            0 => Groceries,
            1 => OtherFood,
            2 => GiftsForOthers,
            3 => GiftsForSelf,
            4 => Clothes,
            5 => Car,
            6 => Rent,
            7 => Travel,
            8 => Misc,
            9 => Subscriptions,
            _ => unreachable!(), // This case will never be hit because we are generating numbers from 0 to 9.
        }
    }

    pub fn _get_all() -> Vec<Category> {
        Category::iter().collect()
    }
}
