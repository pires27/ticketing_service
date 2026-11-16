# Assumptions

## Inventory

### Stations and routes
- Routes are unidirectional. Each direction is a different route, closer to reality.
- Each route has "at least 2 stops". These are intermediate stops and do not include origin and destination.
- Station and routes data is mocked for the assignment purpose, covering suggested scenarios.
- Each stop records the distance in kms from the previous stop.
- Distances are stored as the assignment requires but are not used by any feature yet.

Route 1: Paris - London. Stops:  Lille - Calais
Route 2: Paris - Amsterdam. Stops:  Brussels - Rotterdam
Route 3: Amsterdam - Berlin. Stops: Cologne - Hamburg
Route 4: London - Amsterdam. Stops: Calais - Paris - Brussels

### Services
- A service is identified by its number and date.
- Every service runs once per day, every day. No indication of time frames. So, queries work for any service and date.
- Each route is operated by one service. Service 5160 (from the assignment) corresponds to route 4 in the mocked data, the other services also have fixed mocked numbers for consistency.

### Carriages and seats
- Each service has 4 carriages: A, H, N and T. These are labels. Keeping it simple and including the ones mentioned in the assignment scenarios. It is also common for train operators to bypass letters when labeling carriages so realism is not too compromised.
- Half of the carriages is first class (A and H) and the other half is second class (N and T). One comfort zone per carriage.
- First class carriages hold 40 seats each. Second class carriages hold 60 seats. Making each service be able to hold a total capacity of 200 passengers.
- A seat is unique per service and has exactly one comfort zone. Identified by a carriage letter and a number. Mock data uses one zone per carriage but the model allows mixed carriages also.

### Booking
- A seat is only occupied between a ticket's origin and destination. Occupancy checks are ticket based.
- Booking is all-or-nothing, if a seat is unavailable or invalid booking fails.
- Multiple tickets in the same booking reservation must not hold the same seat in the given range of stops.
- Specified comfort zone when making a reservation is validated against the seat.
- In assignment proposed scenario 3: assumption is that passengers change seats at Paris (initial seats were not available for the whole journey), H1 then A1 first class, N5 then T7 second class. If the assumption was changing trains, it is also supported since each ticket references its own service.

## Out of scope
- Prices, products and payments
- Automatic seat assignment
- Storing data, all data is in memory.
