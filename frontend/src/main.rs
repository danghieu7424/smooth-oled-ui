// src/main.rs
use leptos::*;

mod app;
use crate::app::App;

mod features;
mod pages;
mod shared;
mod store;
mod utils;

fn main() {
    mount_to_body(|| view! { <App /> })
}