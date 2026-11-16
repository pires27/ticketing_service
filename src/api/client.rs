use crate::domain::booking::{Booking, BookingId};
use crate::domain::inventory::InventoryError;
use crate::mock_data;
use crate::reservation::system::{BookingError, BookingRequest, ReservationSystem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status_code: u16,
    pub body: ResponseBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseBody {
    Booking(Booking),
    Error(String),
}

/// REST-like interface
pub trait HttpClient {
    fn post(&mut self, url: &str, body: BookingRequest) -> Response;
    fn get(&self, url: &str) -> Response;
}

/// Calls the reservation system directly (no HTTP in between).
#[derive(Debug)]
pub struct InMemoryClient {
    system: ReservationSystem,
}

impl InMemoryClient {
    pub fn new(system: ReservationSystem) -> Self {
        Self { system }
    }

    pub fn with_mock_data() -> Result<Self, InventoryError> {
        Ok(Self::new(ReservationSystem::new(mock_data::services()?)?))
    }
}

impl HttpClient for InMemoryClient {
    fn post(&mut self, url: &str, body: BookingRequest) -> Response {
        if url != "/bookings" {
            return Response::error(404, "not found");
        }
        match self.system.book(body) {
            Ok(id) => match self.system.booking(id) {
                Some(booking) => Response::booking(201, booking.clone()),
                None => Response::error(500, "booking was not stored"),
            },
            Err(error) => Response::error(status_code(error), error.to_string()),
        }
    }

    fn get(&self, url: &str) -> Response {
        let Some(id) = url.strip_prefix("/bookings/") else {
            return Response::error(404, "not found");
        };
        let Some(id) = parse_booking_id(id) else {
            return Response::error(400, "booking id must be a number");
        };
        match self.system.booking(id) {
            Some(booking) => Response::booking(200, booking.clone()),
            None => Response::error(404, "booking not found"),
        }
    }
}

impl Response {
    fn booking(status_code: u16, booking: Booking) -> Self {
        Self {
            status_code,
            body: ResponseBody::Booking(booking),
        }
    }

    fn error(status_code: u16, message: impl Into<String>) -> Self {
        Self {
            status_code,
            body: ResponseBody::Error(message.into()),
        }
    }
}

fn parse_booking_id(text: &str) -> Option<BookingId> {
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok().map(BookingId)
}

fn status_code(error: BookingError) -> u16 {
    match error {
        BookingError::EmptyRequest
        | BookingError::ComfortZoneMismatch
        | BookingError::InvalidJourney(_) => 400,
        BookingError::UnknownService | BookingError::UnknownSeat => 404,
        BookingError::SeatUnavailable(_) => 409,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::booking::{Passenger, PassengerReservation, Ticket};
    use crate::domain::date::Date;
    use crate::domain::inventory::{ComfortZone, SeatId, ServiceId, Station};

    fn client() -> InMemoryClient {
        InMemoryClient::with_mock_data().unwrap()
    }

    /// First-class seat A11 on service 5160 from Paris to Amsterdam.
    fn ticket() -> Ticket {
        Ticket {
            service: ServiceId {
                number: 5160,
                date: Date::new(2021, 4, 1).unwrap(),
            },
            seat: SeatId {
                carriage: 'A',
                number: 11,
            },
            comfort_zone: ComfortZone::FirstClass,
            origin: Station::Paris,
            destination: Station::Amsterdam,
        }
    }

    fn request(ticket: Ticket) -> BookingRequest {
        BookingRequest {
            reservations: vec![PassengerReservation {
                passenger: Passenger {
                    name: "Alice".to_string(),
                },
                tickets: vec![ticket],
            }],
        }
    }

    fn booking(response: Response) -> Booking {
        match response.body {
            ResponseBody::Booking(booking) => booking,
            ResponseBody::Error(message) => panic!("expected a booking, got: {message}"),
        }
    }

    #[test]
    fn post_valid_booking_is_201() {
        let response = client().post("/bookings", request(ticket()));
        assert_eq!(response.status_code, 201);
        assert_eq!(booking(response).id(), BookingId(1));
    }

    #[test]
    fn post_same_request_again_is_409() {
        let mut client = client();
        client.post("/bookings", request(ticket()));
        assert_eq!(
            client.post("/bookings", request(ticket())),
            Response::error(409, "seat A11 is not available on this journey")
        );
    }

    #[test]
    fn post_comfort_zone_mismatch_is_400() {
        let ticket = Ticket {
            comfort_zone: ComfortZone::SecondClass,
            ..ticket()
        };
        assert_eq!(client().post("/bookings", request(ticket)).status_code, 400);
    }

    #[test]
    fn post_unknown_service_is_404() {
        let ticket = Ticket {
            service: ServiceId {
                number: 9999,
                ..ticket().service
            },
            ..ticket()
        };
        assert_eq!(client().post("/bookings", request(ticket)).status_code, 404);
    }

    #[test]
    fn get_existing_booking_is_200() {
        let mut client = client();
        let created = booking(client.post("/bookings", request(ticket())));
        assert_eq!(client.get("/bookings/1"), Response::booking(200, created));
    }

    #[test]
    fn get_unknown_booking_is_404() {
        assert_eq!(client().get("/bookings/99").status_code, 404);
    }

    #[test]
    fn get_non_numeric_booking_id_is_400() {
        assert_eq!(client().get("/bookings/abc").status_code, 400);
    }

    #[test]
    fn get_booking_id_with_plus_sign_is_400() {
        let mut client = client();
        client.post("/bookings", request(ticket()));
        assert_eq!(client.get("/bookings/+1").status_code, 400);
    }

    #[test]
    fn unknown_route_is_404() {
        let mut client = client();
        assert_eq!(client.get("/trains").status_code, 404);
        assert_eq!(client.post("/trains", request(ticket())).status_code, 404);
    }
}
