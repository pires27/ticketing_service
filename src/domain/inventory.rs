use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use std::num::NonZeroU8;
use std::ops::Range;

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
    TooFewStops,
    DuplicateStation,
    InvalidCarriageId,
    DuplicateSeat,
    NoCarriages,
    DuplicateCarriage,
    DuplicateService,
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewStops => write!(f, "a route needs at least {MIN_STOPS} stops"),
            Self::DuplicateStation => f.write_str("a station appears more than once on the route"),
            Self::InvalidCarriageId => {
                f.write_str("carriage ID must be an uppercase letter A to Z")
            }
            Self::DuplicateSeat => {
                f.write_str("a seat number appears more than once in the carriage")
            }
            Self::NoCarriages => f.write_str("a service needs at least one carriage"),
            Self::DuplicateCarriage => {
                f.write_str("a carriage ID appears more than once on the service")
            }
            Self::DuplicateService => f.write_str("a service number appears more than once"),
        }
    }
}

impl Error for InventoryError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteError {
    StationNotOnRoute,
    InvalidDirection,
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StationNotOnRoute => "station is not on this service's route",
            Self::InvalidDirection => "destination must come after origin on this route",
        })
    }
}

impl Error for RouteError {}

/// Returns true if any item appears more than once.
fn has_duplicates<T: Eq + Hash>(items: impl IntoIterator<Item = T>) -> bool {
    let mut seen = HashSet::new();
    items.into_iter().any(|item| !seen.insert(item))
}

/// Identifies a seat within a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeatId {
    pub carriage: char,
    pub number: NonZeroU8,
}

/// Carriage letter followed by seat number.
impl fmt::Display for SeatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.carriage, self.number)
    }
}

/// Identifies a service running on a given date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceId {
    pub number: u32,
    pub date: Date,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub number: NonZeroU8,
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
            return Err(InventoryError::TooFewStops);
        }
        if has_duplicates(stops.iter().map(|stop| stop.station)) {
            return Err(InventoryError::DuplicateStation);
        }
        Ok(Self { stops })
    }

    pub fn stops(&self) -> &[Stop] {
        &self.stops
    }

    pub fn journey(
        &self,
        origin: Station,
        destination: Station,
    ) -> Result<Range<usize>, RouteError> {
        let position = |station| {
            self.stops
                .iter()
                .position(|stop| stop.station == station)
                .ok_or(RouteError::StationNotOnRoute)
        };
        let (start, end) = (position(origin)?, position(destination)?);
        if start >= end {
            return Err(RouteError::InvalidDirection);
        }
        Ok(start..end)
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

    pub fn seat(&self, id: SeatId) -> Option<&Seat> {
        self.carriages
            .iter()
            .find(|carriage| carriage.id == id.carriage)?
            .seats
            .iter()
            .find(|seat| seat.number == id.number)
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
            number: NonZeroU8::new(number).unwrap(),
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
        assert_eq!(Route::new(stops).unwrap_err(), InventoryError::TooFewStops);
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
    fn journey_between_stations() {
        assert_eq!(route().journey(Station::Lille, Station::London), Ok(1..3));
    }

    #[test]
    fn journey_to_same_station() {
        assert_eq!(
            route().journey(Station::Lille, Station::Lille),
            Err(RouteError::InvalidDirection)
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
    fn seat_lookup() {
        let service = Service::new(5160, route(), vec![carriage('A')]).unwrap();
        let id = |carriage, number| SeatId {
            carriage,
            number: NonZeroU8::new(number).unwrap(),
        };
        assert_eq!(service.seat(id('A', 2)), Some(&seat(2)));
        assert_eq!(service.seat(id('H', 1)), None);
        assert_eq!(service.seat(id('A', 3)), None);
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
