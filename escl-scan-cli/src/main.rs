/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate clap;
extern crate scan;

use clap::Parser;
use scan::scanner::Scanner;
use std::path::Path;
use std::process::exit;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enables overwriting the output file if it already exists
    #[arg(short = 'f', long = "force")]
    overwrite: bool,

    /// Scan resolution in DPI (Dots Per Inch)
    #[arg(short = 'r', long = "resolution", default_value = "75")]
    dpi: i16,

    /// IP or hostname of the scanner
    #[arg(long = "host", required = true)]
    host: String,

    /// Print scanner info and exit
    #[arg(short, long)]
    info: bool,

    /// Output file name
    #[arg(value_name = "OUTPUT_FILE_NAME", default_value = "scan.jpg")]
    output_file_name: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    if !args.overwrite && Path::new(&args.output_file_name).exists() {
        eprintln!("Output file exists, exiting...");
        exit(1);
    }

    let scanner = Scanner::new(args.host, None);
    match scanner.get_status() {
        Ok(state) => println!("Scanner state: {state}"),
        Err(err) => {
            eprintln!("Failed to get status: {err:?}");
            exit(1);
        }
    }

    // TODO This is just a band-aid for testing
    if args.info {
        exit(0);
    }

    if let Err(err) = scanner.scan(args.dpi, &args.output_file_name) {
        eprintln!("Failed to scan: {err:?}");
        exit(1);
    }
}
