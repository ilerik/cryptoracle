use std::io;
use std::time::{Duration, Instant};
use futures::{Future, Stream};
use futures::future::Either;
use hyper;
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use tokio_core::reactor::Timeout;
use serde_json;
use serde_json::Value;

use errors::*;

pub enum Currency {
    Bitcoin(String),
    Ethereum(String)
}

#[derive(Debug, Deserialize)]
pub struct HistoItem {
    time: u64,
    high: f64,
    low: f64,
    open: f64,
    volumefrom: f64,
    volumeto: f64,
    close: f64,
}

#[derive(Debug, Deserialize)]
pub struct HistoResponse {
    Response: bool,
    Type: u64,
    Aggregated: bool,
    Data: Vec<HistoItem>,
    TimeTo: u64,
    TimeFrom: u64,
}

/// Fetch aggregated historical data given market
pub fn fetch_market_history(timeout: u64, time_step: u64, time_points: u64) -> Result<HistoResponse> {
    let response = HistoResponse {
        Response: false,
        Type: 0,
        Aggregated: false,
        Data: Vec::new(),
        TimeTo: 0,
        TimeFrom: 0,
    };

    // Spin up event loop
    let mut core = Core::new()?;
    let handle = core.handle();

    // Prepare client
    let client = Client::configure()
    .connector(HttpsConnector::new(4, &handle)?)
    .build(&handle);
    
    // Prepare request and parse response
    let uri = "https://min-api.cryptocompare.com/data/histominute?fsym=BTC&tsym=ETH&limit=20&aggregate=10&e=CCCAGG".parse()?;
    let get = client.get(uri).and_then(|res| {
        println!("Response status: {}", res.status());
        res.body().concat2().and_then(move |body| {
            let v: Value = serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            })?;
            println!("Response content: {}", v["Data"]);
            Ok(response)
        })
    });

    // Combine with timeout 
    let timeout = Timeout::new(Duration::from_secs(timeout), &handle)?;
    let work = get.select2(timeout).then(|res| match res {
        Ok(Either::A((got, _timeout))) => Ok(got),
        Ok(Either::B((_timeout_error, _get))) => {
            Err(hyper::Error::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "Client timed out while connecting",
            )))
        }
        Err(Either::A((get_error, _timeout))) => Err(get_error),
        Err(Either::B((timeout_error, _get))) => Err(From::from(timeout_error)),
    });

    Ok(core.run(work)?)
}