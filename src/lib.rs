extern crate chrono;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate reqwest;
#[macro_use] extern crate error_chain;

#[cfg(test)] extern crate dotenv;

mod utils;
pub mod error;
pub mod client;
pub mod products;
pub mod orders;