/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate clap;
extern crate scan;

use clap::{Args, Parser};
use scan::scanner::Scanner;
use scan::scannerfinder::ScannerFinder;
use std::path::Path;
use std::process::exit;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    device: DeviceArgs,

    /// Overwrite the output file if it already exists
    #[arg(short = 'f', long = "force")]
    overwrite: bool,

    /// Scan resolution in DPI (Dots Per Inch)
    #[arg(short = 'r', long = "resolution", default_value = "300")]
    dpi: i16,

    /// Output file name
    #[arg(value_name = "OUTPUT_FILE_NAME", default_value = "scan.jpg")]
    output_file_name: String,
}

#[derive(Args)]
struct DeviceArgs {
    /// Select scanner by IP or hostname
    #[arg(long = "host")]
    host: Option<String>,

    /// Select scanner by device name (can be partial)
    #[arg(long, short)]
    name: Option<String>,

    /// Print information about the scanner identified by device name
    #[arg(short, long)]
    info: Option<String>,

    /// List available scanners
    #[arg(short, long)]
    list: bool,
}

fn list_scanners() {
    let mut finder = ScannerFinder::new();
    let scanners = match finder.find(None) {
        Ok(scanners) => scanners,
        Err(err) => {
            eprintln!("Failed to discover scanners: {err}");
            exit(1);
        }
    };

    if scanners.len() == 0 {
        println!("No scanners found");
    } else if scanners.len() == 1 {
        println!("Found 1 scanner:");
    } else {
        println!("Found {} scanners:", scanners.len());
    }

    for scanner in scanners {
        println!("{scanner}");
    }
}

fn get_scanner(cli: &Cli) -> Result<Scanner, String> {
    if let Some(host) = &cli.device.host {
        return Ok(Scanner::new("Manually Configured", &host, None));
    }

    let scanners = match ScannerFinder::new().find(cli.device.name.as_deref()) {
        Ok(scanners) => scanners,
        Err(err) => return Err(err.to_string()),
    };

    if let Some(scanner) = scanners.first() {
        return Ok(scanner.clone());
    }

    return Err("No scanners found".to_string());
}

fn main() {
    env_logger::init();
    let args = Cli::parse();

    if args.device.list {
        list_scanners();
        exit(0);
    }

    // TODO This is just a band-aid for testing
    if let Some(name) = args.device.info {
        exit(0);
    }

    let scanner = match get_scanner(&args) {
        Ok(scanner) => scanner,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

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
