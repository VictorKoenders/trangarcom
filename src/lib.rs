#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

pub mod models;
pub mod schema;
