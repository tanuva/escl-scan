/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate reqwest;
extern crate serde;
extern crate serde_xml_rs;

use crate::{
    scannererror::{ErrorCode, ScannerError},
    structs,
};
use reqwest::blocking::Response;
use std::{fmt::Display, fs::File};

#[derive(Clone, Debug)]
pub struct Scanner {
    pub base_url: String,
    pub device_name: String,
    pub capabilities: structs::ScannerCapabilities,
}

impl Scanner {
    fn make_base_url(ip_or_host: &str, root: &str) -> String {
        format!("http://{}:80/{}", ip_or_host, root)
    }

    fn get_capabilities(base_url: &str) -> Result<structs::ScannerCapabilities, ScannerError> {
        let response = match reqwest::blocking::get(&format!("{}/ScannerCapabilities", base_url)) {
            Ok(response) => response,
            Err(err) => return Err(err.into()),
        };

        let response_string = response.text().expect("text is a string");
        log::debug!("> Capabilities: {response_string}");
        let scanner_capabilities: structs::ScannerCapabilities =
            match serde_xml_rs::from_str(&response_string) {
                Ok(caps) => caps,
                Err(err) => return Err(err.into()),
            };
        Ok(scanner_capabilities)
    }

    pub fn new(
        device_name: &str,
        ip_or_host: &str,
        resource_root: &str,
    ) -> Result<Scanner, ScannerError> {
        let base_url = Scanner::make_base_url(ip_or_host, resource_root);
        let capabilities = match Self::get_capabilities(&base_url) {
            Ok(caps) => caps,
            Err(err) => return Err(err),
        };

        Ok(Scanner {
            device_name: device_name.to_string(),
            base_url,
            capabilities,
        })
    }

    pub fn get_status(&self) -> Result<structs::ScannerState, ScannerError> {
        log::info!("Getting scanner status");
        let response = match reqwest::blocking::get(&format!("{}/ScannerStatus", self.base_url)) {
            Ok(response) => response,
            Err(err) => return Err(err.into()),
        };
        log::debug!("ScannerStatus: {:?}", response);

        let response_string = response.text().expect("text is a string");
        log::debug!("ScannerStatus: {:?}", response_string);

        let scanner_status: structs::ScannerStatus = match serde_xml_rs::from_str(&response_string)
        {
            Ok(status) => status,
            Err(err) => return Err(err.into()),
        };

        log::info!("Scanner state: {}", scanner_status.state);
        Ok(scanner_status.state)
    }

    pub fn make_settings(&self) -> structs::ScanSettings {
        structs::ScanSettings {
            version: "2.6".to_string(),
            scan_regions: structs::ScanRegion {
                x_offset: 0,
                y_offset: 0,
                width: self.capabilities.platen.platen_input_caps.max_width,
                height: self.capabilities.platen.platen_input_caps.max_height,
                content_region_units: "escl:ThreeHundredthsOfInches".to_string(),
            },
            input_source: "Platen".to_string(),
            color_mode: "RGB24".to_string(),
            document_format: "image/jpeg".to_string(),
            x_resolution: 300,
            y_resolution: 300,
        }
    }

    pub fn scan(
        &self,
        scan_settings: &structs::ScanSettings,
        destination_file: &str,
    ) -> Result<(), ScannerError> {
        let request_body = match serde_xml_rs::to_string(scan_settings) {
            Ok(body) => body,
            Err(err) => return Err(err.into()),
        };

        log::info!("Sending scan request with settings: {:?}", scan_settings);
        let scan_response = match self.get_scan_response(request_body) {
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
        return self.download_page(&download_url, destination_file);
    }

    fn get_scan_response(&self, request_body: String) -> Result<Response, ScannerError> {
        let client = reqwest::blocking::Client::new();
        let request = client
            .post(format!("{}/ScanJobs", &self.base_url).as_str())
            .body(format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>{}",
                request_body
            ));
        log::debug!("< ScanJobs: {request:#?}\nBody: {request_body:#?}");
        let response = match request.send() {
            Ok(response) => response,
            Err(err) => return Err(err.into()),
        };
        log::debug!("> ScanJobs: {response:#?}");

        if !response.status().is_success() {
            return Err(ScannerError {
                code: ErrorCode::NetworkError,
                message: format!("{response:?}"),
            });
        }

        return Ok(response);
    }

    fn download_page(
        &self,
        download_url: &str,
        destination_file: &str,
    ) -> Result<(), ScannerError> {
        // We need to try downloadng at least once again, expecting a 404, to make
        // sure we got everything.
        // This is necessary on my Brother MFC-L2710DW to get it to idle state
        // again. It will wait for timeout otherwise, even if we got the scanned
        // page earlier.
        let mut page: u16 = 1;
        loop {
            log::info!("Downloading page {page} to {destination_file}");
            let mut response = match reqwest::blocking::get(download_url) {
                Ok(response) => response,
                Err(err) => return Err(err.into()),
            };

            if response.status() == 404 {
                log::info!("There is no page {page}, we're done");
                break;
            }

            let mut file = match File::create(destination_file) {
                Ok(file) => file,
                Err(err) => return Err(err.into()),
            };

            if let Err(err) = response.copy_to(&mut file) {
                return Err(err.into());
            }

            page += 1;
        }

        Ok(())
    }
}

impl Display for Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}
- URL: {}
- Capabilities: {:#?}",
            self.device_name, self.base_url, self.capabilities
        )
    }
}
