use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::fmt;

use crate::domain::booking::{Booking, BookingId, PassengerReservation, Ticket};
use crate::domain::inventory::{InventoryError, Route, RouteError, SeatId, Service};

use super::occupancy::conflicts;

/// An unvalidated booking request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookingRequest {
    pub reservations: Vec<PassengerReservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookingError {
    EmptyRequest,
    UnknownService,
    UnknownSeat,
    ComfortZoneMismatch,
    InvalidJourney(RouteError),
    SeatUnavailable(SeatId),
}

impl fmt::Display for BookingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRequest => {
                f.write_str("a booking needs at least one passenger, each with at least one ticket")
            }
            Self::UnknownService => f.write_str("service not found"),
            Self::UnknownSeat => f.write_str("seat not found on this service"),
            Self::ComfortZoneMismatch => {
                f.write_str("requested comfort zone does not match the seat")
            }
            Self::InvalidJourney(error) => error.fmt(f),
            Self::SeatUnavailable(seat) => {
                write!(f, "seat {seat} is not available on this journey")
            }
        }
    }
}

impl Error for BookingError {}

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
    pub fn new(services: Vec<Service>) -> Result<Self, InventoryError> {
        let mut by_number = HashMap::new();
        for service in services {
            match by_number.entry(service.number()) {
                Entry::Occupied(_) => return Err(InventoryError::DuplicateService),
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

        let tickets = reservations
            .iter()
            .flat_map(|r| &r.tickets)
            .map(|ticket| self.validate_ticket(ticket).map(|route| (ticket, route)))
            .collect::<Result<Vec<_>, _>>()?;

        for (i, &(ticket, route)) in tickets.iter().enumerate() {
            let earlier = tickets[..i].iter().map(|&(earlier, _)| earlier);
            self.check_seat_available(ticket, route, earlier)?;
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
    /// Returns the service's route for the availability check.
    fn validate_ticket(&self, ticket: &Ticket) -> Result<&Route, BookingError> {
        let service = self
            .services
            .get(&ticket.service.number)
            .ok_or(BookingError::UnknownService)?;
        let seat = service.seat(ticket.seat).ok_or(BookingError::UnknownSeat)?;
        if seat.comfort_zone != ticket.comfort_zone {
            return Err(BookingError::ComfortZoneMismatch);
        }
        service.route().journey(ticket.origin, ticket.destination)?;
        Ok(service.route())
    }

    fn check_seat_available<'a>(
        &'a self,
        ticket: &Ticket,
        route: &Route,
        earlier: impl Iterator<Item = &'a Ticket>,
    ) -> Result<(), BookingError> {
        let booked = self
            .bookings
            .values()
            .flat_map(|booking| booking.reservations())
            .flat_map(|reservation| &reservation.tickets);
        for other in booked.chain(earlier) {
            if conflicts(ticket, other, route)? {
                return Err(BookingError::SeatUnavailable(ticket.seat));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use super::*;
    use crate::domain::booking::Passenger;
    use crate::domain::date::Date;
    use crate::domain::inventory::{
        Carriage, ComfortZone, Route, Seat, SeatId, ServiceId, Station, Stop,
    };
    use Station::*;

    const SERVICE_NUMBER: u32 = 5160;

    /// Service 5160 on route 4 (London - Amsterdam) with first-class carriages
    /// A and H (40 seats) and second-class carriages N and T (60 seats).
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
                    number: NonZeroU8::new(number).unwrap(),
                    comfort_zone,
                })
                .collect();
            Carriage::new(id, seats).unwrap()
        };
        let carriages = vec![
            carriage('A', 40, ComfortZone::FirstClass),
            carriage('H', 40, ComfortZone::FirstClass),
            carriage('N', 60, ComfortZone::SecondClass),
            carriage('T', 60, ComfortZone::SecondClass),
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
                number: NonZeroU8::new(seat).unwrap(),
            },
            comfort_zone: ComfortZone::FirstClass,
            origin: Paris,
            destination: Amsterdam,
        }
    }

    /// Ticket on service 5160 for any seat, comfort zone and journey.
    fn ticket_between(
        carriage: char,
        seat: u8,
        comfort_zone: ComfortZone,
        origin: Station,
        destination: Station,
    ) -> Ticket {
        Ticket {
            comfort_zone,
            origin,
            destination,
            ..ticket(SERVICE_NUMBER, carriage, seat)
        }
    }

    fn seat_id(carriage: char, number: u8) -> SeatId {
        SeatId {
            carriage,
            number: NonZeroU8::new(number).unwrap(),
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

    fn scenario_1_reservations() -> Vec<PassengerReservation> {
        vec![
            reservation("Alice", vec![ticket(SERVICE_NUMBER, 'A', 11)]),
            reservation("Bob", vec![ticket(SERVICE_NUMBER, 'A', 12)]),
        ]
    }

    /// London - Amsterdam, changing seats at Paris (see ASSUMPTIONS.md).
    fn scenario_3_reservations() -> Vec<PassengerReservation> {
        use ComfortZone::*;
        vec![
            reservation(
                "Alice",
                vec![
                    ticket_between('H', 1, FirstClass, London, Paris),
                    ticket_between('A', 1, FirstClass, Paris, Amsterdam),
                ],
            ),
            reservation(
                "Bob",
                vec![
                    ticket_between('N', 5, SecondClass, London, Paris),
                    ticket_between('T', 7, SecondClass, Paris, Amsterdam),
                ],
            ),
        ]
    }

    #[test]
    fn scenario_1_two_passengers_paris_to_amsterdam() {
        let mut system = system();
        let reservations = scenario_1_reservations();

        let id = system.book(request(reservations.clone())).unwrap();

        let booking = system.booking(id).unwrap();
        assert_eq!(booking.id(), id);
        assert_eq!(booking.reservations(), reservations.as_slice());
    }

    #[test]
    fn scenario_2_repeat_scenario_1() {
        let mut system = system();
        assert_eq!(
            system.book(request(scenario_1_reservations())),
            Ok(BookingId(1))
        );
        assert_eq!(
            system.book(request(scenario_1_reservations())),
            Err(BookingError::SeatUnavailable(seat_id('A', 11)))
        );
        assert!(system.booking(BookingId(2)).is_none());
    }

    #[test]
    fn scenario_3_changing_seats_at_paris() {
        let mut system = system();
        assert_eq!(
            system.book(request(scenario_3_reservations())),
            Ok(BookingId(1))
        );
    }

    #[test]
    fn scenario_4_repeat_scenario_3() {
        let mut system = system();
        assert_eq!(
            system.book(request(scenario_3_reservations())),
            Ok(BookingId(1))
        );
        assert_eq!(
            system.book(request(scenario_3_reservations())),
            Err(BookingError::SeatUnavailable(seat_id('H', 1)))
        );
        assert!(system.booking(BookingId(2)).is_none());
    }

    #[test]
    fn same_seat_on_touching_journeys_in_separate_bookings() {
        let mut system = system();
        let first = ticket_between('A', 1, ComfortZone::FirstClass, London, Paris);
        let second = ticket_between('A', 1, ComfortZone::FirstClass, Paris, Amsterdam);
        assert_eq!(book_one(&mut system, first), Ok(BookingId(1)));
        assert_eq!(book_one(&mut system, second), Ok(BookingId(2)));
    }

    #[test]
    fn same_seat_and_journey_on_a_different_date() {
        let mut system = system();
        let today = ticket(SERVICE_NUMBER, 'A', 1);
        let tomorrow = Ticket {
            service: ServiceId {
                number: SERVICE_NUMBER,
                date: Date::new(2021, 4, 2).unwrap(),
            },
            ..today
        };
        assert_eq!(book_one(&mut system, today), Ok(BookingId(1)));
        assert_eq!(book_one(&mut system, tomorrow), Ok(BookingId(2)));
    }

    #[test]
    fn same_seat_on_overlapping_journeys_within_one_request() {
        let mut system = system();
        let request = request(vec![
            reservation(
                "Alice",
                vec![ticket_between(
                    'A',
                    1,
                    ComfortZone::FirstClass,
                    London,
                    Brussels,
                )],
            ),
            reservation(
                "Bob",
                vec![ticket_between(
                    'A',
                    1,
                    ComfortZone::FirstClass,
                    Paris,
                    Amsterdam,
                )],
            ),
        ]);
        assert_eq!(
            system.book(request),
            Err(BookingError::SeatUnavailable(seat_id('A', 1)))
        );
        assert!(system.booking(BookingId(1)).is_none());
    }

    #[test]
    fn same_seat_on_touching_journeys_within_one_request() {
        let mut system = system();
        let request = request(vec![
            reservation(
                "Alice",
                vec![ticket_between(
                    'A',
                    1,
                    ComfortZone::FirstClass,
                    London,
                    Paris,
                )],
            ),
            reservation(
                "Bob",
                vec![ticket_between(
                    'A',
                    1,
                    ComfortZone::FirstClass,
                    Paris,
                    Amsterdam,
                )],
            ),
        ]);
        assert_eq!(system.book(request), Ok(BookingId(1)));
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
            InventoryError::DuplicateService
        );
    }
}
