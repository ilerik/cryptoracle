use std::fs::File;
use std::io::prelude::*;
use serde::de::{Deserialize, DeserializeOwned};

use errors::*;
use toml;

pub fn from_file<T>(mut f: File) -> Result<T>
    where T: DeserializeOwned
{
    let mut contents = String::new();
    f.read_to_string(&mut contents);
    let result: T = toml::from_str(&contents)?;
    Ok(result)
}

#[derive(Debug, Deserialize)]
/// Public part of configuration
pub struct Configuration {
    mongodb_addr: String,
}

#[derive(Debug, Deserialize)]
// Secret part of configuration
pub struct Secrets {
    bittrex_token: String,
}