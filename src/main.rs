mod api;
mod app;
mod models;
mod routes;

fn main() {
    dioxus::launch(app::App);
}
