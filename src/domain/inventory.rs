#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Station {
    London,
    Calais,
    Lille,
    Paris,
    Brussels,
    Rotterdam,
    Amsterdam,
    Cologne,
    Hamburg,
    Berlin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComfortZone {
    FirstClass,
    SecondClass,
}

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

/// Identifies a seat within a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeatId {
    pub carriage: char,
    pub number: u8,
}

/// Identifies a service running on a given date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceIdentifier {
    pub number: u32,
    pub date: Date,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub number: u8,
    pub comfort_zone: ComfortZone,
}

#[derive(Debug, Clone)]
pub struct Carriage {
    id: char,
    seats: Vec<Seat>,
}

impl Carriage {
    pub fn id(&self) -> char {
        self.id
    }

    pub fn seats(&self) -> &[Seat] {
        &self.seats
    }
}

#[derive(Debug, Clone)]
pub struct Stop {
    pub station: Station,
    pub distance_from_previous_km: u32,
}

#[derive(Debug, Clone)]
pub struct Route {
    stops: Vec<Stop>,
}

impl Route {
    pub fn stops(&self) -> &[Stop] {
        &self.stops
    }
}

#[derive(Debug, Clone)]
pub struct Service {
    number: u32,
    route: Route,
    carriages: Vec<Carriage>,
}

impl Service {
    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn carriages(&self) -> &[Carriage] {
        &self.carriages
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
