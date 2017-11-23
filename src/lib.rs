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
extern crate toml;
#[macro_use(bson, doc)] extern crate bson;
extern crate mongodb;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

pub mod config;
pub mod errors;

use futures::future;
use futures::{Future, Stream};
use tokio_core::reactor::Core;

use bson::Bson;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use errors::*;

// Entrypoint
pub fn run() -> Result<()> {
    // Some preparations and configuration steps
    pretty_env_logger::init()?;

    // Create the event loop that will drive this server
    let mut core = Core::new()?;
    let handle = core.handle();

        let client = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone client.");

    let coll = client.db("test").collection("movies");

    let doc = doc! { 
        "title": "Jaws",
        "array": [ 1, 2, 3 ],
    };

    // Insert document into 'test.movies' collection
    coll.insert_one(doc.clone(), None)
        .ok().expect("Failed to insert document.");

    // Find the document and receive a cursor
    let mut cursor = coll.find(Some(doc.clone()), None)
        .ok().expect("Failed to execute find.");

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
    
    // Process data
    // core.run(future::done(Ok(|| {
    //     // Fetch historical data on BTC\ETH rates etc

    //     // And put it in MongoDB for storage

    //     // Train prediction model

    //     // Make predictions about future prices

    //     // And put results back


    // })
    // .map( |_| Ok(()) )
    // .map_err( |e| "Error" )
    // ))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}