/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate reqwest;
extern crate serde;
extern crate serde_xml_rs;

pub mod scannererror;
pub mod structs;

use crate::scannererror::ErrorCode;
use crate::scannererror::ScannerError;
use reqwest::blocking::Response;
use std::fs::File;

pub fn scan(
    scanner_base_path: &str,
    scan_resolution: i16,
    destination_file: &str,
) -> Result<(), ScannerError> {
    log::info!("Getting scanner capabilities...");
    let scanner_capabilities = match get_scanner_capabilities(&scanner_base_path) {
        Ok(caps) => caps,
        Err(err) => return Err(err),
    };

    let scan_settings: structs::ScanSettings = structs::ScanSettings {
        version: "2.6".to_string(),
        scan_regions: structs::ScanRegion {
            x_offset: 0,
            y_offset: 0,
            width: scanner_capabilities.platen.platen_input_caps.max_width,
            height: scanner_capabilities.platen.platen_input_caps.max_height,
            content_region_units: "escl:ThreeHundredthsOfInches".to_string(),
        },
        input_source: "Platen".to_string(),
        color_mode: "RGB24".to_string(),
        x_resolution: scan_resolution,
        y_resolution: scan_resolution,
    };

    let request_body = match serde_xml_rs::to_string(&scan_settings) {
        Ok(body) => body,
        Err(err) => return Err(err.into()),
    };

    log::info!("Sending scan request with DPI {}...", scan_resolution);
    let scan_response = match get_scan_response(scanner_base_path, request_body) {
        Ok(response) => response,
        Err(err) => return Err(err),
    };
    let location = match scan_response.headers().get("location") {
        Some(location) => location.to_str().expect("'location' can be a string"),
        None => {
            return Err(ScannerError {
                code: ErrorCode::ProtocolError,
                message: format!(
                    "Failed to get 'location' header from response:\n{scan_response:?}"
                ),
            });
        }
    };

    let download_url = format!("{}/NextDocument", location);
    return download_scan(&download_url, destination_file);
}

pub fn get_scanner_capabilities(
    scanner_base_path: &str,
) -> Result<structs::ScannerCapabilities, ScannerError> {
    let response =
        match reqwest::blocking::get(&format!("{}/ScannerCapabilities", scanner_base_path)) {
            Ok(response) => response,
            Err(err) => return Err(err.into()),
        };

    let response_string = response.text().expect("text is a string");
    let scanner_capabilities: structs::ScannerCapabilities =
        match serde_xml_rs::from_str(&response_string) {
            Ok(caps) => caps,
            Err(err) => return Err(err.into()),
        };
    Ok(scanner_capabilities)
}

fn get_scan_response(
    scanner_base_path: &str,
    request_body: String,
) -> Result<Response, ScannerError> {
    let client = reqwest::blocking::Client::new();
    let request = client
        .post(format!("{}/ScanJobs", &scanner_base_path).as_str())
        .body(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>{}",
            request_body
        ));
    let response = match request.send() {
        Ok(response) => response,
        Err(err) => return Err(err.into()),
    };

    if !response.status().is_success() {
        return Err(ScannerError {
            code: ErrorCode::NetworkError,
            message: format!("{response:?}"),
        });
    }

    return Ok(response);
}

fn download_scan(download_url: &str, destination_file: &str) -> Result<(), ScannerError> {
    log::info!("Downloading output file to {}...", destination_file);
    let mut response = match reqwest::blocking::get(download_url) {
        Ok(response) => response,
        Err(err) => return Err(err.into()),
    };

    let mut file = match File::create(destination_file) {
        Ok(file) => file,
        Err(err) => return Err(err.into()),
    };

    if let Err(err) = response.copy_to(&mut file) {
        return Err(err.into());
    }

    Ok(())
}
