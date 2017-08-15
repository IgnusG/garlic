#![allow(unknown_lints)]
#![allow(too_many_arguments)]
#![allow(dead_code)]
#![feature(conservative_impl_trait)]

extern crate simplelog;
#[macro_use]
extern crate log;
extern crate getopts;
#[macro_use]
extern crate error_chain;
extern crate colored;
extern crate stoppable_thread;
extern crate mio;
extern crate case;
extern crate bytes;
extern crate byteorder;
#[macro_use]
extern crate structure;
extern crate bit_field;
#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

// Required modules
#[macro_use]
mod errors;
mod config;
mod messages;
mod brunch;
mod core;

// This is the import order for all modules
// Crate Imports
use getopts::Options;
use simplelog::{CombinedLogger, TermLogger};
// Standard Imports
use std::env;
use std::panic;
// Custom Imports
use errors::*;

/** Specification of cmd arguments **/
fn parse_cmd_arguments() -> Result<getopts::Matches> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.reqopt("c", "config", "set path for the config file", "");
    opts.parse(&args[1..]).chain_err(|| "couldn't parse arguments")
}

/** Eval cmd arguments and initialize methods, bootstrap brunch **/
fn bootstrap() -> Result<()> {
    let arguments = parse_cmd_arguments()?;

    let config_file_path = arguments.opt_str("c").unwrap();

    let conf = config::read_config_file(config_file_path)
        .chain_err(|| "couldn't create configuration struct")?;

    brunch::start(conf)
}

/** Setup logger and boostrap the app **/
fn main() {
    panic::set_hook(Box::new(|e| {
        use colored::*;
        use case::CaseExt;

        if let Some(e) = e.payload().downcast_ref::<Error>() {
            let mut s = format!("{}: {}", "Critical Problem".red().bold(),
                &format!("{}", e).to_capitalized());
            for e in e.iter().skip(1) {
                s.push_str(&format!("\n → {}: {}", "Caused by".bold().dimmed(),
                    &format!("{}", e).to_capitalized()));
            };
            error!("{}", s);
        } else if let Some(e) = e.payload().downcast_ref::<&str>() {
            println!("{}", e.red().bold());
        } else {
            error!("{}", "App panicked but the error was malformed (most likely a bug)".red().bold());
            ::std::process::exit(1);
        }
        ::std::process::exit(2);
    }));

    CombinedLogger::init(
        vec![
            TermLogger::new(log::LogLevelFilter::Info, simplelog::Config {
                time: Some(log::LogLevel::Warn),
                level: None, target: None, location: None
            }).expect("Failed to initialize terminal logger")
        ]
    ).expect("Failed to initialize logger");

    trace_panic! { bootstrap()? };
}
