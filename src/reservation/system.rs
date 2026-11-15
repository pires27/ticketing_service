use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::domain::booking::{Booking, BookingId, PassengerReservation, Ticket};
use crate::domain::inventory::{RouteError, Service};

/// An unvalidated booking request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookingRequest {
    pub reservations: Vec<PassengerReservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookingError {
    EmptyRequest,
    UnknownService,
    DuplicateService,
    UnknownSeat,
    ComfortZoneMismatch,
    InvalidJourney(RouteError),
}

impl From<RouteError> for BookingError {
    fn from(error: RouteError) -> Self {
        Self::InvalidJourney(error)
    }
}

#[derive(Debug)]
pub struct ReservationSystem {
    services: HashMap<u32, Service>,
    bookings: HashMap<BookingId, Booking>,
    next_booking_id: u64,
}

impl ReservationSystem {
    /// Inventory is loaded once here and is read-only afterwards.
    /// Only bookings change at runtime. Rejects duplicate service numbers.
    pub fn new(services: Vec<Service>) -> Result<Self, BookingError> {
        let mut by_number = HashMap::new();
        for service in services {
            match by_number.entry(service.number()) {
                Entry::Occupied(_) => return Err(BookingError::DuplicateService),
                Entry::Vacant(slot) => {
                    slot.insert(service);
                }
            }
        }
        Ok(Self {
            services: by_number,
            bookings: HashMap::new(),
            next_booking_id: 1,
        })
    }

    pub fn book(&mut self, request: BookingRequest) -> Result<BookingId, BookingError> {
        let reservations = request.reservations;

        if reservations.is_empty() || reservations.iter().any(|r| r.tickets.is_empty()) {
            return Err(BookingError::EmptyRequest);
        }

        for ticket in reservations.iter().flat_map(|r| &r.tickets) {
            self.validate_ticket(ticket)?;
        }

        let id = BookingId(self.next_booking_id);
        self.next_booking_id += 1;
        self.bookings.insert(id, Booking::new(id, reservations));
        Ok(id)
    }

    pub fn booking(&self, id: BookingId) -> Option<&Booking> {
        self.bookings.get(&id)
    }

    /// Checks a ticket against the inventory: service, seat, comfort zone, journey.
    fn validate_ticket(&self, ticket: &Ticket) -> Result<(), BookingError> {
        let service = self
            .services
            .get(&ticket.service.number)
            .ok_or(BookingError::UnknownService)?;
        let seat = service.seat(ticket.seat).ok_or(BookingError::UnknownSeat)?;
        if seat.comfort_zone != ticket.comfort_zone {
            return Err(BookingError::ComfortZoneMismatch);
        }
        service.route().journey(ticket.origin, ticket.destination)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::booking::Passenger;
    use crate::domain::date::Date;
    use crate::domain::inventory::{
        Carriage, ComfortZone, Route, Seat, SeatId, ServiceId, Station, Stop,
    };
    use Station::*;

    const SERVICE_NUMBER: u32 = 5160;

    /// Service 5160 on route 4 (London - Amsterdam) with carriages A and N.
    fn service() -> Service {
        let stop = |station, distance_from_previous_km| Stop {
            station,
            distance_from_previous_km,
        };
        let route = Route::new(vec![
            stop(London, 0),
            stop(Calais, 150),
            stop(Paris, 290),
            stop(Brussels, 310),
            stop(Amsterdam, 210),
        ])
        .unwrap();
        let carriage = |id, seat_count, comfort_zone| {
            let seats = (1..=seat_count)
                .map(|number| Seat {
                    number,
                    comfort_zone,
                })
                .collect();
            Carriage::new(id, seats).unwrap()
        };
        let carriages = vec![
            carriage('A', 40, ComfortZone::FirstClass),
            carriage('N', 60, ComfortZone::SecondClass),
        ];
        Service::new(SERVICE_NUMBER, route, carriages).unwrap()
    }

    fn system() -> ReservationSystem {
        ReservationSystem::new(vec![service()]).unwrap()
    }

    /// First-class ticket from Paris to Amsterdam on 2021-04-01.
    fn ticket(service_number: u32, carriage: char, seat: u8) -> Ticket {
        Ticket {
            service: ServiceId {
                number: service_number,
                date: Date::new(2021, 4, 1).unwrap(),
            },
            seat: SeatId {
                carriage,
                number: seat,
            },
            comfort_zone: ComfortZone::FirstClass,
            origin: Paris,
            destination: Amsterdam,
        }
    }

    fn reservation(name: &str, tickets: Vec<Ticket>) -> PassengerReservation {
        PassengerReservation {
            passenger: Passenger {
                name: name.to_string(),
            },
            tickets,
        }
    }

    fn request(reservations: Vec<PassengerReservation>) -> BookingRequest {
        BookingRequest { reservations }
    }

    fn book_one(system: &mut ReservationSystem, ticket: Ticket) -> Result<BookingId, BookingError> {
        system.book(request(vec![reservation("Alice", vec![ticket])]))
    }

    #[test]
    fn scenario_1_two_passengers_paris_to_amsterdam() {
        let mut system = system();
        let reservations = vec![
            reservation("Alice", vec![ticket(SERVICE_NUMBER, 'A', 11)]),
            reservation("Bob", vec![ticket(SERVICE_NUMBER, 'A', 12)]),
        ];

        let id = system.book(request(reservations.clone())).unwrap();

        let booking = system.booking(id).unwrap();
        assert_eq!(booking.id(), id);
        assert_eq!(booking.reservations(), reservations.as_slice());
    }

    #[test]
    fn booking_ids_are_sequential() {
        let mut system = system();
        let first = system.book(request(vec![reservation(
            "Alice",
            vec![ticket(SERVICE_NUMBER, 'A', 1)],
        )]));
        let second = system.book(request(vec![reservation(
            "Bob",
            vec![ticket(SERVICE_NUMBER, 'A', 2)],
        )]));
        assert_eq!(first, Ok(BookingId(1)));
        assert_eq!(second, Ok(BookingId(2)));
    }

    #[test]
    fn failed_booking_stores_nothing_and_keeps_next_id() {
        let mut system = system();
        let failing = request(vec![reservation("Alice", vec![ticket(9999, 'A', 11)])]);
        assert_eq!(system.book(failing), Err(BookingError::UnknownService));
        assert!(system.booking(BookingId(1)).is_none());

        let valid = request(vec![reservation(
            "Alice",
            vec![ticket(SERVICE_NUMBER, 'A', 11)],
        )]);
        assert_eq!(system.book(valid), Ok(BookingId(1)));
    }

    #[test]
    fn unknown_service() {
        let mut system = system();
        let request = request(vec![reservation("Alice", vec![ticket(9999, 'A', 11)])]);
        assert_eq!(system.book(request), Err(BookingError::UnknownService));
    }

    #[test]
    fn unknown_seat() {
        let mut system = system();
        assert_eq!(
            book_one(&mut system, ticket(SERVICE_NUMBER, 'A', 99)),
            Err(BookingError::UnknownSeat)
        );
    }

    #[test]
    fn second_class_seat_requested_as_first_class() {
        let mut system = system();
        assert_eq!(
            book_one(&mut system, ticket(SERVICE_NUMBER, 'N', 5)),
            Err(BookingError::ComfortZoneMismatch)
        );
    }

    #[test]
    fn station_not_on_route() {
        let mut system = system();
        let ticket = Ticket {
            destination: Berlin,
            ..ticket(SERVICE_NUMBER, 'A', 1)
        };
        assert_eq!(
            book_one(&mut system, ticket),
            Err(BookingError::InvalidJourney(RouteError::StationNotOnRoute))
        );
    }

    #[test]
    fn backwards_journey() {
        let mut system = system();
        let ticket = Ticket {
            origin: Amsterdam,
            destination: Paris,
            ..ticket(SERVICE_NUMBER, 'A', 1)
        };
        assert_eq!(
            book_one(&mut system, ticket),
            Err(BookingError::InvalidJourney(RouteError::InvalidDirection))
        );
    }

    #[test]
    fn one_invalid_passenger_fails_the_whole_booking() {
        let mut system = system();
        let request = request(vec![
            reservation("Alice", vec![ticket(SERVICE_NUMBER, 'A', 1)]),
            reservation("Bob", vec![ticket(SERVICE_NUMBER, 'A', 99)]),
        ]);
        assert_eq!(system.book(request), Err(BookingError::UnknownSeat));
        assert!(system.booking(BookingId(1)).is_none());
    }

    #[test]
    fn empty_request() {
        let mut system = system();
        let no_passengers = request(vec![]);
        let passenger_without_tickets = request(vec![reservation("Alice", vec![])]);
        assert_eq!(system.book(no_passengers), Err(BookingError::EmptyRequest));
        assert_eq!(
            system.book(passenger_without_tickets),
            Err(BookingError::EmptyRequest)
        );
    }

    #[test]
    fn duplicate_service() {
        assert_eq!(
            ReservationSystem::new(vec![service(), service()]).unwrap_err(),
            BookingError::DuplicateService
        );
    }
}
