use chrono::NaiveDate;
use std::collections::HashMap;

/// Модель студента с часами пропусков по датам.
/// Дополнительно можно указывать «уважительные» часы по дням.
#[derive(Debug, Clone)]
pub struct Student {
    pub name: String,
    pub hours_by_date: HashMap<NaiveDate, u32>,
    pub excused_by_date: HashMap<NaiveDate, u32>, // «У»
}

impl Student {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hours_by_date: HashMap::new(),
            excused_by_date: HashMap::new(),
        }
    }

    pub fn add_hours(&mut self, date: NaiveDate, hours: u32) {
        *self.hours_by_date.entry(date).or_insert(0) += hours;
    }

    pub fn add_excused_hours(&mut self, date: NaiveDate, hours: u32) {
        *self.excused_by_date.entry(date).or_insert(0) += hours;
    }
}
