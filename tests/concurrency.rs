use std::num::NonZeroU8;
use std::sync::{Arc, Mutex};
use std::thread;

use ticketing_service::{
    BookingRequest, ComfortZone, Date, HttpClient, InMemoryClient, Passenger, PassengerReservation,
    SeatId, ServiceId, Station, Ticket,
};

const THREADS: usize = 8;

/// One passenger in first-class seat A11 on service 5160, Paris to Amsterdam, 2021-04-01.
fn request_for_a11(name: &str) -> BookingRequest {
    BookingRequest {
        reservations: vec![PassengerReservation {
            passenger: Passenger {
                name: name.to_string(),
            },
            tickets: vec![Ticket {
                service: ServiceId {
                    number: 5160,
                    date: Date::new(2021, 4, 1).expect("valid date"),
                },
                seat: SeatId {
                    carriage: 'A',
                    number: NonZeroU8::new(11).expect("seat numbers start at 1"),
                },
                comfort_zone: ComfortZone::FirstClass,
                origin: Station::Paris,
                destination: Station::Amsterdam,
            }],
        }],
    }
}

#[test]
fn only_one_thread_books_the_same_seat() {
    let client = Arc::new(Mutex::new(
        InMemoryClient::with_mock_data().expect("mock data is valid"),
    ));

    let handles: Vec<_> = (0..THREADS)
        .map(|i| {
            let client = Arc::clone(&client);
            thread::spawn(move || {
                // The lock is held for the whole post, so no other booking can happen between the check and the save.
                let mut client = client.lock().expect("lock is not poisoned");
                client
                    .post("/bookings", request_for_a11(&format!("Passenger {i}")))
                    .status_code
            })
        })
        .collect();

    let mut status_codes: Vec<u16> = handles
        .into_iter()
        .map(|handle| handle.join().expect("thread did not panic"))
        .collect();
    status_codes.sort();

    let mut expected = vec![409; THREADS - 1];
    expected.insert(0, 201);
    assert_eq!(status_codes, expected);
}
