use std::collections::HashSet;
use std::hash::Hash;

use super::date::Date;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryError {
    InvalidRoute,
    DuplicateStation,
    InvalidCarriageId,
    InvalidSeatNumber,
    DuplicateSeat,
    NoCarriages,
    DuplicateCarriage,
    StationNotOnRoute,
    InvalidDirection,
}

/// Returns true if any item appears more than once.
fn has_duplicates<T: Eq + Hash>(items: impl IntoIterator<Item = T>) -> bool {
    let mut seen = HashSet::new();
    items.into_iter().any(|item| !seen.insert(item))
}

/// Identifies a seat within a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeatId {
    pub carriage: char,
    pub number: u8,
}

/// Identifies a service running on a given date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceId {
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
    pub fn new(id: char, seats: Vec<Seat>) -> Result<Self, InventoryError> {
        if !id.is_ascii_uppercase() {
            return Err(InventoryError::InvalidCarriageId);
        }

        if seats.iter().any(|seat| seat.number == 0) {
            return Err(InventoryError::InvalidSeatNumber);
        }
        if has_duplicates(seats.iter().map(|seat| seat.number)) {
            return Err(InventoryError::DuplicateSeat);
        }
        Ok(Self { id, seats })
    }

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

/// Origin + destination + minimum intermediate stops.
const MIN_STOPS: usize = 4;

impl Route {
    pub fn new(stops: Vec<Stop>) -> Result<Self, InventoryError> {
        if stops.len() < MIN_STOPS {
            return Err(InventoryError::InvalidRoute);
        }
        if has_duplicates(stops.iter().map(|stop| stop.station)) {
            return Err(InventoryError::DuplicateStation);
        }
        Ok(Self { stops })
    }

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
    pub fn new(
        number: u32,
        route: Route,
        carriages: Vec<Carriage>,
    ) -> Result<Self, InventoryError> {
        if carriages.is_empty() {
            return Err(InventoryError::NoCarriages);
        }
        if has_duplicates(carriages.iter().map(|carriage| carriage.id)) {
            return Err(InventoryError::DuplicateCarriage);
        }
        Ok(Self {
            number,
            route,
            carriages,
        })
    }

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

    fn stop(station: Station) -> Stop {
        Stop {
            station,
            distance_from_previous_km: 100,
        }
    }

    fn seat(number: u8) -> Seat {
        Seat {
            number,
            comfort_zone: ComfortZone::FirstClass,
        }
    }

    fn route() -> Route {
        use Station::*;
        Route::new(vec![stop(Paris), stop(Lille), stop(Calais), stop(London)]).unwrap()
    }

    fn carriage(id: char) -> Carriage {
        Carriage::new(id, vec![seat(1), seat(2)]).unwrap()
    }

    #[test]
    fn valid_route() {
        assert_eq!(route().stops().len(), 4);
    }

    #[test]
    fn invalid_route_stops() {
        let stops = vec![
            stop(Station::Paris),
            stop(Station::Lille),
            stop(Station::London),
        ];
        assert_eq!(Route::new(stops).unwrap_err(), InventoryError::InvalidRoute);
    }

    #[test]
    fn route_with_duplicate_station() {
        use Station::*;
        let stops = vec![stop(Paris), stop(Lille), stop(Paris), stop(London)];
        assert_eq!(
            Route::new(stops).unwrap_err(),
            InventoryError::DuplicateStation
        );
    }

    #[test]
    fn valid_carriage() {
        let carriage = carriage('A');
        assert_eq!((carriage.id(), carriage.seats().len()), ('A', 2));
    }

    #[test]
    fn carriage_with_invalid_id() {
        for id in ['a', '1', 'É'] {
            assert_eq!(
                Carriage::new(id, vec![seat(1)]).unwrap_err(),
                InventoryError::InvalidCarriageId
            );
        }
    }

    #[test]
    fn carriage_with_seat_number_zero() {
        assert_eq!(
            Carriage::new('A', vec![seat(0)]).unwrap_err(),
            InventoryError::InvalidSeatNumber
        );
    }

    #[test]
    fn carriage_with_duplicate_seat() {
        assert_eq!(
            Carriage::new('A', vec![seat(1), seat(1)]).unwrap_err(),
            InventoryError::DuplicateSeat
        );
    }

    #[test]
    fn valid_service() {
        let service = Service::new(5160, route(), vec![carriage('A'), carriage('H')]).unwrap();
        assert_eq!((service.number(), service.carriages().len()), (5160, 2));
    }

    #[test]
    fn service_with_no_carriages() {
        assert_eq!(
            Service::new(5160, route(), vec![]).unwrap_err(),
            InventoryError::NoCarriages
        );
    }

    #[test]
    fn service_with_duplicate_carriage() {
        assert_eq!(
            Service::new(5160, route(), vec![carriage('A'), carriage('A')]).unwrap_err(),
            InventoryError::DuplicateCarriage
        );
    }
}
