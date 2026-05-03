mod state;
mod errors;
mod fairings;
mod catchers;
mod guards;
mod request;
mod routes;

#[rocket::launch]
fn rocket() -> _ {
    // TODO: wire all dependencies here
    rocket::build()
}
