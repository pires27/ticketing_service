pub mod api;
pub mod domain;
pub mod mock_data;
pub mod reservation;

pub use api::client::{HttpClient, InMemoryClient, Response, ResponseBody};
pub use domain::booking::{Booking, BookingId, Passenger, PassengerReservation, Ticket};
pub use domain::date::Date;
pub use domain::inventory::{ComfortZone, SeatId, ServiceId, Station};
pub use reservation::system::BookingRequest;
