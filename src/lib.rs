#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod editor;
mod features;
mod platform;
mod ui;

pub use app::{TextEditApp, TextEditAppState};
