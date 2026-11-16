use std::ops::Range;

use crate::domain::booking::Ticket;
use crate::domain::inventory::{Route, RouteError};

pub(super) fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

/// Checks if two tickets conflict: same service and date, same seat, and overlapping journeys on the route.
pub(super) fn conflicts(a: &Ticket, b: &Ticket, route: &Route) -> Result<bool, RouteError> {
    if a.service != b.service || a.seat != b.seat {
        return Ok(false);
    }
    let a_journey = route.journey(a.origin, a.destination)?;
    let b_journey = route.journey(b.origin, b.destination)?;
    Ok(overlaps(&a_journey, &b_journey))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touching_ranges_do_not_overlap() {
        assert!(!overlaps(&(0..2), &(2..4)));
    }

    #[test]
    fn partial_overlap() {
        assert!(overlaps(&(0..3), &(2..4)));
    }

    #[test]
    fn one_range_inside_the_other() {
        assert!(overlaps(&(0..4), &(1..2)));
    }

    #[test]
    fn identical_ranges() {
        assert!(overlaps(&(1..3), &(1..3)));
    }

    #[test]
    fn disjoint_ranges() {
        assert!(!overlaps(&(0..1), &(3..4)));
    }
}
