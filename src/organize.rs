use strum_macros::EnumIter;

#[derive(serde::Deserialize, serde::Serialize, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum GroupBy {
    Day,
    Month,
    Year,
}

impl std::fmt::Display for GroupBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            GroupBy::Day => write!(f, "Day"),
            GroupBy::Month => write!(f, "Month"),
            GroupBy::Year => write!(f, "Year"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, EnumIter, PartialEq, Eq, Copy, Clone)]
pub enum SortBy {
    Date,
    Cost,
}

impl std::fmt::Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SortBy::Cost => write!(f, "Cost"),
            SortBy::Date => write!(f, "Date"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, EnumIter, PartialEq, Eq, Copy, Clone)]
pub enum SortOrder {
    Increasing,
    Decreasing,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SortOrder::Increasing => write!(f, "Increasing"),
            SortOrder::Decreasing => write!(f, "Decreasing"),
        }
    }
}
