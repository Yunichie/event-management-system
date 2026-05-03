pub mod events;
pub mod bookings;
pub mod tickets;
pub mod refunds;

use rocket::Route;

/// Collects all API routes from all sub-modules.
pub fn all_routes() -> Vec<Route> {
    // TODO: mount all routes here
    vec![]
}
