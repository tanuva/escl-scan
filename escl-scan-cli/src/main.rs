/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate clap;
extern crate scan;

use clap::{Args, Parser};
use scan::scanner::Scanner;
use std::path::Path;
use std::process::exit;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    device: DeviceArgs,

    /// Enables overwriting the output file if it already exists
    #[arg(short = 'f', long = "force")]
    overwrite: bool,

    /// Scan resolution in DPI (Dots Per Inch)
    #[arg(short = 'r', long = "resolution", default_value = "75")]
    dpi: i16,

    /// Output file name
    #[arg(value_name = "OUTPUT_FILE_NAME", default_value = "scan.jpg")]
    output_file_name: String,
}

#[derive(Args)]
#[group(required = true)]
struct DeviceArgs {
    /// Select scanner by IP or hostname
    #[arg(long = "host")]
    host: Option<String>,

    /// Print information about the scanner identified by device name
    #[arg(short, long)]
    info: Option<String>,
}

fn get_scanner(cli: &Cli) -> Scanner {
    if let Some(host) = &cli.device.host {
        return Scanner::new(&host, None);
    }

    panic!("get_scanner called while no device was specified");
}

fn main() {
    env_logger::init();
    let args = Cli::parse();

    // TODO This is just a band-aid for testing
    if let Some(name) = args.device.info {
        exit(0);
    }

    let scanner = get_scanner(&args);

    if !args.overwrite && Path::new(&args.output_file_name).exists() {
        eprintln!("Output file exists, exiting...");
        exit(1);
    }

    match scanner.get_status() {
        Ok(state) => println!("Scanner state: {state}"),
        Err(err) => {
            eprintln!("Failed to get status: {err:?}");
            exit(1);
        }
    }

    if let Err(err) = scanner.scan(args.dpi, &args.output_file_name) {
        eprintln!("Failed to scan: {err:?}");
        exit(1);
    }
}
