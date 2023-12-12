/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate clap;
extern crate scan;

use clap::{Args, Parser, ValueEnum};
use scan::scanner::Scanner;
use scan::scannerfinder::ScannerFinder;
use scan::structs::{self};
use std::process::exit;

#[derive(Clone, ValueEnum)]
enum CliColorMode {
    BlackAndWhite,
    Grayscale,
    RGB,
}

impl From<CliColorMode> for String {
    fn from(value: CliColorMode) -> Self {
        match value {
            CliColorMode::BlackAndWhite => "BlackAndWhite1".to_string(),
            CliColorMode::Grayscale => "Grayscale8".to_string(),
            CliColorMode::RGB => "RGB24".to_string(),
        }
    }
}

#[derive(Clone, ValueEnum)]
enum CliOutputFormat {
    JPG,
    PDF,
}

impl From<CliOutputFormat> for String {
    fn from(value: CliOutputFormat) -> Self {
        match value {
            CliOutputFormat::JPG => "image/jpeg".to_string(),
            CliOutputFormat::PDF => "application/pdf".to_string(),
        }
    }
}

#[derive(Clone, ValueEnum)]
enum CliDocumentSize {
    A4Portrait,
    A5Landscape,
    A5Portrait,
    USLetterPortrait,
}

impl From<CliDocumentSize> for structs::ScanRegion {
    fn from(value: CliDocumentSize) -> Self {
        match value {
            CliDocumentSize::A4Portrait => structs::ScanRegion::a4_portrait(),
            CliDocumentSize::A5Landscape => structs::ScanRegion::a5_landscape(),
            CliDocumentSize::A5Portrait => structs::ScanRegion::a5_portrait(),
            CliDocumentSize::USLetterPortrait => structs::ScanRegion::us_letter_portrait(),
        }
    }
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    device: DeviceArgs,

    /// Input document format
    #[arg(short, long, value_enum, default_value = "a4-portrait")]
    input_format: CliDocumentSize,

    /// Scan resolution in DPI (Dots Per Inch)
    #[arg(short = 'r', long = "resolution", default_value = "300")]
    dpi: i16,

    /// Output file name
    #[arg(value_name = "OUTPUT_FILE_NAME", default_value = "scan.jpg")]
    output_file_name: String,

    /// Output document format
    #[arg(short, long, value_enum, default_value = "jpg")]
    output_format: CliOutputFormat,

    /// Color mode
    #[arg(short, long, value_enum, default_value = "rgb")]
    color: CliColorMode,
}

#[derive(Args)]
struct DeviceArgs {
    /// Select scanner by IP or hostname
    #[arg(long = "host")]
    host: Option<String>,

    /// Select scanner by device name (can be partial)
    #[arg(long, short)]
    name: Option<String>,

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
        return match Scanner::new("Manually Configured", &host, "eSCL") {
            Ok(scanner) => Ok(scanner),
            Err(err) => Err(format!("{err}")),
        };
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

    let scanner = match get_scanner(&args) {
        Ok(scanner) => scanner,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

    match scanner.get_status() {
        Ok(state) => println!("Scanner state: {state}"),
        Err(err) => {
            eprintln!("Failed to get status: {err:?}");
            exit(1);
        }
    }

    let mut scan_settings = scanner.make_settings();
    scan_settings.x_resolution = args.dpi;
    scan_settings.y_resolution = args.dpi;
    scan_settings.color_mode = args.color.into();
    scan_settings.document_format = args.output_format.into();
    scan_settings.scan_regions = args.input_format.into();
    scan_settings.feed_direction = structs::FeedDirection::ShortEdgeFeed.into();

    if let Err(err) = scanner.scan(&scan_settings, &args.output_file_name) {
        eprintln!("Failed to scan: {err:?}");
        exit(1);
    }
}
