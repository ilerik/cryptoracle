// allow modern futures impl Trait mechanics
#![feature(conservative_impl_trait)]
// enable <'_> construction (claimed unstable)
#![feature(underscore_lifetimes)]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
extern crate pretty_env_logger;
extern crate log;
extern crate tokio_core;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate toml;
#[macro_use(bson, doc)] extern crate bson;
extern crate mongodb;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

pub mod config;
pub mod errors;
pub mod clients;

use futures::future;
use futures::{Future, Stream};
use tokio_core::reactor::{Core, Handle};

use bson::Bson;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use errors::*;

use clients::cryptocompare::fetch_market_history;

// Entrypoint
pub fn run() -> Result<()> {
    // Fetch historical data from cryptocompare
    let fetch_data = future::done(fetch_market_history(5, 1, 1));

    // Connect to mongodb
    let client = Client::connect("localhost", 27017)
        .chain_err(|| "Failed to initialize standalone client.")?;

    let coll = client.db("data").collection("btc_eth");

    let doc = doc! { 
        "title": "Jaws",
        "btc_eth": [ 1, 2, 3 ],
    };

    // Insert document into 'test.movies' collection
    coll.insert_one(doc.clone(), None)
        .ok().chain_err(|| "Failed to insert document.")?;

    // Find the document and receive a cursor
    let mut cursor = coll.find(Some(doc.clone()), None)
        .ok().chain_err(|| "Failed to execute find.")?;

    let item = cursor.next();

    // cursor.next() returns an Option<Result<Document>>
    match item {
        Some(Ok(doc)) => match doc.get("title") {
            Some(&Bson::String(ref title)) => println!("{}", title),
            _ => panic!("Expected title to be a string!"),
        },
        Some(Err(_)) => panic!("Failed to get next from server!"),
        None => panic!("Server returned no results!"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}