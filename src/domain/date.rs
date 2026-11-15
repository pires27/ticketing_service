/// Calendar date. Fields are ordered: year, month, day
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateError {
    InvalidMonth,
    InvalidDay,
}

impl Date {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, DateError> {
        if !(1..=12).contains(&month) {
            return Err(DateError::InvalidMonth);
        }
        if day == 0 || day > days_in_month(year, month) {
            return Err(DateError::InvalidDay);
        }
        Ok(Self { year, month, day })
    }

    pub fn year(&self) -> u16 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn day(&self) -> u8 {
        self.day
    }
}

fn is_leap_year(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_date() {
        let date = Date::new(2021, 4, 1).unwrap();
        assert_eq!((date.year(), date.month(), date.day()), (2021, 4, 1));
    }

    #[test]
    fn invalid_month() {
        assert_eq!(Date::new(2021, 13, 1), Err(DateError::InvalidMonth));
    }

    #[test]
    fn invalid_day() {
        assert_eq!(Date::new(2021, 5, 0), Err(DateError::InvalidDay));
    }

    #[test]
    fn invalid_day_in_month() {
        assert_eq!(Date::new(2021, 4, 31), Err(DateError::InvalidDay));
    }

    #[test]
    fn leap_day_in_leap_year() {
        assert!(Date::new(2020, 2, 29).is_ok());
    }

    #[test]
    fn leap_day_in_non_leap_year() {
        assert_eq!(Date::new(2021, 2, 29), Err(DateError::InvalidDay));
    }
}
