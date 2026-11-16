use ticketing_service::{
    Booking, BookingId, BookingRequest, ComfortZone, Date, HttpClient, InMemoryClient, Passenger,
    PassengerReservation, Response, ResponseBody, SeatId, ServiceId, Station, Ticket,
};

use ComfortZone::*;
use Station::*;

fn client() -> InMemoryClient {
    InMemoryClient::with_mock_data().expect("mock data is valid")
}

fn on_april_1st(number: u32) -> ServiceId {
    ServiceId {
        number,
        date: Date::new(2021, 4, 1).expect("valid date"),
    }
}

fn ticket(
    carriage: char,
    seat: u8,
    comfort_zone: ComfortZone,
    origin: Station,
    destination: Station,
) -> Ticket {
    Ticket {
        service: on_april_1st(5160),
        seat: SeatId {
            carriage,
            number: seat,
        },
        comfort_zone,
        origin,
        destination,
    }
}

/// Booking request body
fn booking_request(passengers: Vec<(&str, Vec<Ticket>)>) -> BookingRequest {
    BookingRequest {
        reservations: passengers
            .into_iter()
            .map(|(name, tickets)| PassengerReservation {
                passenger: Passenger {
                    name: name.to_string(),
                },
                tickets,
            })
            .collect(),
    }
}

/// Two passengers from Paris to Amsterdam in first-class seats A11 and A12.
fn scenario_1() -> BookingRequest {
    booking_request(vec![
        ("Alice", vec![ticket('A', 11, FirstClass, Paris, Amsterdam)]),
        ("Bob", vec![ticket('A', 12, FirstClass, Paris, Amsterdam)]),
    ])
}

/// Two passengers from London to Amsterdam, changing seats at Paris:
/// H1 then A1 in first class, N5 then T7 in second class.
fn scenario_3() -> BookingRequest {
    booking_request(vec![
        (
            "Alice",
            vec![
                ticket('H', 1, FirstClass, London, Paris),
                ticket('A', 1, FirstClass, Paris, Amsterdam),
            ],
        ),
        (
            "Bob",
            vec![
                ticket('N', 5, SecondClass, London, Paris),
                ticket('T', 7, SecondClass, Paris, Amsterdam),
            ],
        ),
    ])
}

fn booking(response: Response) -> Booking {
    match response.body {
        ResponseBody::Booking(booking) => booking,
        ResponseBody::Error(message) => panic!("expected a booking, got: {message}"),
    }
}

#[test]
fn scenario_1_two_passengers_paris_to_amsterdam() {
    let mut client = client();

    let response = client.post("/bookings", scenario_1());

    assert_eq!(response.status_code, 201);
    let booking = booking(response);
    assert_eq!(booking.id(), BookingId(1));
    assert_eq!(booking.reservations().len(), 2);
}

#[test]
fn scenario_2_same_seats_again_is_rejected() {
    let mut client = client();
    assert_eq!(client.post("/bookings", scenario_1()).status_code, 201);

    let response = client.post("/bookings", scenario_1());

    assert_eq!(response.status_code, 409);
    assert_eq!(client.get("/bookings/2").status_code, 404);
}

#[test]
fn scenario_3_london_to_amsterdam_changing_seats_at_paris() {
    let mut client = client();

    let response = client.post("/bookings", scenario_3());

    assert_eq!(response.status_code, 201);
}

#[test]
fn scenario_4_same_seat_change_again_is_rejected() {
    let mut client = client();
    assert_eq!(client.post("/bookings", scenario_3()).status_code, 201);

    let response = client.post("/bookings", scenario_3());

    assert_eq!(response.status_code, 409);
    assert_eq!(client.get("/bookings/2").status_code, 404);
}
