use std::num::NonZeroU8;

use crate::domain::inventory::{
    Carriage, ComfortZone, InventoryError, Route, Seat, Service, Station, Stop,
};
use Station::*;

pub fn services() -> Result<Vec<Service>, InventoryError> {
    Ok(vec![
        Service::new(
            9014,
            route(&[(Paris, 0), (Lille, 225), (Calais, 110), (London, 165)])?,
            standard_carriages()?,
        )?,
        Service::new(
            9315,
            route(&[
                (Paris, 0),
                (Brussels, 310),
                (Rotterdam, 150),
                (Amsterdam, 75),
            ])?,
            standard_carriages()?,
        )?,
        Service::new(
            140,
            route(&[
                (Amsterdam, 0),
                (Cologne, 260),
                (Hamburg, 430),
                (Berlin, 290),
            ])?,
            standard_carriages()?,
        )?,
        Service::new(
            5160,
            route(&[
                (London, 0),
                (Calais, 165),
                (Paris, 290),
                (Brussels, 310),
                (Amsterdam, 210),
            ])?,
            standard_carriages()?,
        )?,
    ])
}

fn route(stops: &[(Station, u32)]) -> Result<Route, InventoryError> {
    Route::new(
        stops
            .iter()
            .map(|&(station, distance_from_previous_km)| Stop {
                station,
                distance_from_previous_km,
            })
            .collect(),
    )
}

fn single_class_carriage(
    id: char,
    comfort_zone: ComfortZone,
    seats: u8,
) -> Result<Carriage, InventoryError> {
    Carriage::new(
        id,
        (1..=seats)
            .filter_map(NonZeroU8::new)
            .map(|number| Seat {
                number,
                comfort_zone,
            })
            .collect(),
    )
}

/// First class: 40 seats, 2 carriages (A and H). Second class: 60 seats, 2 carriages (N and T).
fn standard_carriages() -> Result<Vec<Carriage>, InventoryError> {
    use ComfortZone::*;
    Ok(vec![
        single_class_carriage('A', FirstClass, 40)?,
        single_class_carriage('H', FirstClass, 40)?,
        single_class_carriage('N', SecondClass, 60)?,
        single_class_carriage('T', SecondClass, 60)?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reservation::system::ReservationSystem;

    fn service(number: u32) -> Service {
        services()
            .unwrap()
            .into_iter()
            .find(|service| service.number() == number)
            .unwrap()
    }

    #[test]
    fn data_is_valid() {
        assert_eq!(services().unwrap().len(), 4);
    }

    #[test]
    fn loads_into_the_system() {
        assert!(ReservationSystem::new(services().unwrap()).is_ok());
    }

    #[test]
    fn service_5160_matches_the_assumptions() {
        let stations: Vec<Station> = service(5160)
            .route()
            .stops()
            .iter()
            .map(|stop| stop.station)
            .collect();
        assert_eq!(stations, [London, Calais, Paris, Brussels, Amsterdam]);
    }

    #[test]
    fn carriages_match_the_assumptions() {
        use ComfortZone::*;
        let service = service(5160);
        let carriages: Vec<(char, ComfortZone, usize)> = service
            .carriages()
            .iter()
            .map(|carriage| {
                let seats = carriage.seats();
                let zone = seats[0].comfort_zone;
                assert!(seats.iter().all(|seat| seat.comfort_zone == zone));
                (carriage.id(), zone, seats.len())
            })
            .collect();
        assert_eq!(
            carriages,
            [
                ('A', FirstClass, 40),
                ('H', FirstClass, 40),
                ('N', SecondClass, 60),
                ('T', SecondClass, 60),
            ]
        );
    }
}
