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
extern crate gnuplot;

pub mod config;
pub mod errors;
pub mod clients;

use futures::future;
use futures::{Future, Stream};
use tokio_core::reactor::{Core, Handle};

use bson::Bson;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use std::io::Write as IoWrite;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use serde::Deserialize;
use gnuplot::{Figure, Caption, Color};

use errors::*;
use clients::cryptocompare::fetch_market_history;
use clients::cryptocompare::HistoItem;

// BSON object we want to store inside mongodb
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DataSet {
    #[serde(rename = "_id")]  // Use MongoDB's special primary key field name when serializing
    pub name: String,
    pub data: Vec<HistoItem>,
    #[serde(with = "bson::compat::u2f")]
    pub time_to: u64,
    #[serde(with = "bson::compat::u2f")]
    pub time_from: u64,
}

// Entrypoint
pub fn run() -> Result<()> {
    // Fetch historical data for last week from cryptocompare 
    // using their great aggregated index (https://www.cryptocompare.com/media/12318004/cccagg.pdf)
    let timespan = 5 * 24 * 60; // 5 days
    let timestep = 60; // 1 hour
    let time_points = timespan / timestep;
    let fetched_data = fetch_market_history(5, timestep, time_points)?;

    // Open a file in write-only mode
    let path = Path::new("out/data.txt");
    let display = path.display();
    let mut file = File::create(&path)?;
    println!("Started dumping raw data to file : {}", display);

    // Write data it to the file on the disk
    for point in fetched_data.data.iter() {        
        write!(&mut file, "{} {} {}\n", 
        point.time,
        point.low,
        point.high)?;
    };
    println!("Raw data has been saved successfully.");

    let dataset_name = String::from("dataset_btc_eth_5d");
    let dataset = DataSet {
        name: dataset_name.clone(),
        data: fetched_data.data,
        time_to: fetched_data.time_to,
        time_from: fetched_data.time_from,
    };

    println!("Connect to MongoDB and upload document with dataset {}", &dataset_name);
    // Connect to mongodb
    let client = Client::connect("localhost", 27017)
        .chain_err(|| "Failed to initialize standalone client.")?;
    let dataset_bson = bson::to_bson(&dataset)?;  // Serialize

    // Insert document into 'cryptocompare.datasets' collection
    let coll = client.db("cryptocompare").collection("datasets"); //
    if let bson::Bson::Document(document) = dataset_bson {
        coll.insert_one(document, None)?;  // Insert into a MongoDB collection
    } else {
        bail!("Error converting the BSON object into a MongoDB document");
    }

    // coll.insert_one(dataset_bson, None)
    //     .ok().chain_err(|| "Failed to insert document.")?;

    // Find the document by _id and receive a cursor
    let doc = doc! { 
        "_id": dataset_name.clone(),
    };
    let mut cursor = coll.find(Some(doc.clone()), None)
        .ok().chain_err(|| "Failed to execute find.")?;

    let item = cursor.next();

    // cursor.next() returns an Option<Result<Document>>
    match item {
        Some(Ok(doc)) => match doc.get("_id") {
            Some(&Bson::String(ref name)) => println!("{}", name),
            _ => panic!("Expected dataset name to be a string!"),
        },
        Some(Err(_)) => panic!("Failed to get next from server!"),
        None => panic!("Server returned no results!"),
    }
    println!("Dataset {} stored successfully to MongoDB", &dataset_name);

    // Ouput plots
    let x = [0u32, 1, 2];
    let y = [3u32, 4, 5];
    let mut fg = Figure::new();
    fg.axes2d()
    .lines(&x, &y, &[Caption("A line"), Color("black")]);
    fg.show();

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}