use std::io;
use std::str::FromStr;
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
use bson;

use errors::*;

pub enum Currency {
    Bitcoin(String),
    Ethereum(String)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoItem {
    #[serde(with = "bson::compat::u2f")]
    pub time: u64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub volumefrom: f64,
    pub volumeto: f64,
    pub close: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HistoResponse {
    pub response: String,
    #[serde(rename = "Type")]
    pub response_type: u64,
    pub aggregated: bool,
    pub data: Vec<HistoItem>,
    pub time_to: u64,
    pub time_from: u64,
}

/// Fetch aggregated historical data given market
pub fn fetch_market_history(timeout: u64, time_step: u64, time_points: u64) -> Result<HistoResponse> {
    // Spin up event loop
    let mut core = Core::new()?;
    let handle = core.handle();

    // Prepare client
    let client = Client::configure()
    .connector(HttpsConnector::new(4, &handle)?)
    .build(&handle);
    
    // Prepare request and parse response
    let uri = format!("https://min-api.cryptocompare.com/data/histominute?fsym={}&tsym={}&limit={}&aggregate={}",
        "BTC",
        "ETH",
        time_points,
        time_step
    ).parse()?;
    let get = client.get(uri).and_then(|res| {
        println!("Response status: {}", res.status());
        res.body().concat2().and_then(move |body| {
            let v: HistoResponse = serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            })?;

            println!("Response contains: {} items", v.data.len());

            Ok(v)
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

#[cfg(test)]
mod tests {
    use errors::*;
    use clients::cryptocompare::HistoResponse;
    use serde_json;
    use serde_json::Value;

    #[test]
    fn test_api() {
        let data = r#"{"Response":"Success","Type":100,"Aggregated":true,"Data":
        [{
            "time":1511610600,
            "high":18.17,
            "low":18.11,
            "open":18.17,
            "volumefrom":106.96,
            "volumeto":1940.67,
            "close":18.12
        },
        {
            "time":1511611200,
            "high":18.15,
            "low":18.12,
            "open":18.12,
            "volumefrom":115.53,
            "volumeto":2095.1500000000007,
            "close":18.14
        },
        {
            "time":1511611800,
            "high":18.15,
            "low":18.12,
            "open":18.14,
            "volumefrom":255.87,
            "volumeto":4643.15,
            "close":18.14
        },
        {
            "time":1511612400,
            "high":18.23,
            "low":18.14,
            "open":18.14,
            "volumefrom":234.71999999999998,
            "volumeto":4272.09,
            "close":18.23
        },
        {
            "time":1511613000,
            "high":18.33,
            "low":18.23,
            "open":18.23,
            "volumefrom":324.35,
            "volumeto":5940.08,
            "close":18.32
        }],
        "TimeTo":1511622900,
        "TimeFrom":1511610600,
        "FirstValueInArray":true,
        "ConversionType": {
            "type":"invert",
            "conversionSymbol":""
            }
        }"#;

        let response: HistoResponse = serde_json::from_str(data).unwrap();
    }
}
