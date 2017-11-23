#![feature(use_extern_macros)]
extern crate cryptoracle;

extern crate futures;
extern crate tokio_core;

use std::rc::Rc;
use std::env;

use futures::future;
use futures::Stream;
use tokio_core::reactor::Core;

use cryptoracle::errors::*;
use cryptoracle::run;

//Entrypoint function that facilitates error handling
fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}