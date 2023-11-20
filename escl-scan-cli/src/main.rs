/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate clap;
extern crate scan;

use clap::Parser;
use std::path::Path;
use std::process::exit;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enables overwriting the output file if it already exists
    #[arg(short = 'f', long = "force")]
    overwrite: bool,

    /// Scan resolution in DPI (Dots Per Inch)
    #[arg(short, long, default_value = "75")]
    dpi: i16,

    /// IP or hostname of the scanner
    #[arg(value_name = "SCANNER_IP")]
    ip: String,

    // , default_value = "scan.jpg"
    /// Output file name
    #[arg(value_name = "OUTPUT_FILE_NAME")]
    output_file_name: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let scanner_base_path = format!("http://{}:80/eSCL", args.ip);

    if !args.overwrite && Path::new(&args.output_file_name).exists() {
        eprintln!("Output file exists, exiting...");
        exit(1);
    }

    if let Err(err) = scan::scan(&scanner_base_path, args.dpi, &args.output_file_name) {
        eprintln!("Failed to scan: {err:?}");
        exit(1);
    }
}
