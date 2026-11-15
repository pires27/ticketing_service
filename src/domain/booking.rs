use super::inventory::{SeatId, ServiceIdentifier, Station};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BookingId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Passenger {
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket {
    pub service: ServiceIdentifier,
    pub seat: SeatId,
    pub origin: Station,
    pub destination: Station,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassengerReservation {
    pub passenger: Passenger,
    pub tickets: Vec<Ticket>,
}

/// A confirmed booking created by the reservation system after validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Booking {
    id: BookingId,
    reservations: Vec<PassengerReservation>,
}

impl Booking {
    pub(crate) fn new(id: BookingId, reservations: Vec<PassengerReservation>) -> Self {
        Self { id, reservations }
    }

    pub fn id(&self) -> BookingId {
        self.id
    }

    pub fn reservations(&self) -> &[PassengerReservation] {
        &self.reservations
    }
}
