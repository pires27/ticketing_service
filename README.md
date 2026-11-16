# Ticketing Service

An in-memory train seat reservation service. Books explicit seats on a train service between an origin and destination for one or more passengers. Rejects bookings already taken on a given part of the itinerary.

The service exposes a REST-like interface without HTTP. Requests are plain method class that return status code and a body.

## Scenarios
The service relies on mocked data: stations, routes, train services, carriages, comfort zones
and seats. The model is versatile enough (while trying to keep it realistic) to support new data and new test scenarios within the given assumptions. Domain naming follows the assignment, so the code is easy to map to it.

The 4 test scenarios from the assignment, all on service 5160 on 2021-04-01:
1. Reservation for two passengers from Paris to Amsterdam in first-class seats A11 and A12.
2. The same reservation again, which fails because the seats are taken.
3. Reservation for two passengers from London to Amsterdam, one in first class and one in second class, changing seats at Paris: H1 then A1 in first class, N5 then T7 in second class.
4. The same reservation again, which fails because the seats are taken.

## How to run

Built and tested with Rust 1.100.0 (2026-11-09). No external dependencies.

```sh
cargo test                      # all unit and scenario tests
cargo test --test scenarios     # only the 4 assignment scenarios
```

The assignment scenarios are in `tests/scenarios.rs`. They use only the public API,
the same way a client would.

## Design

The code is split into three main modules. Dependencies point one way: `api` uses `reservation`, which uses `domain`.

- **domain**: domain definitions for services and bookings
  - `inventory.rs`: types for stations, routes, services, carriages and seats (validation upon creation).
  - `booking.rs`: types for bookings, passengers and tickets.
  - `date.rs`: a simple validated date.
- **reservation**: booking validation and rules
  - `system.rs`: where bookings are created, validates every request before storing anything.
  - `occupancy.rs`: checks for conflicting seat reservations.
- **api**: the REST-like interface
  - `client.rs`: maps URLs to the reservation system and errors to status codes.
- `mock_data.rs`: the sample service data
- `tests/scenarios.rs`: the 4 assignment scenarios


(excalidraw diagram and module overview - to be added)

## Testing

## Assumptions
See [ASSUMPTIONS.md](ASSUMPTIONS.md)

