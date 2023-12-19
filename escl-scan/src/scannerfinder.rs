/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{
    any::Any,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use zeroconf::{
    browser::TMdnsBrowser, event_loop::TEventLoop, txt_record::TTxtRecord, MdnsBrowser,
    ServiceDiscovery, ServiceType, TxtRecord,
};

use crate::{
    scanner::Scanner,
    scannererror::{ErrorCode, ScannerError},
};

pub struct ScannerFinder {
    scanners: Arc<Mutex<Vec<Scanner>>>,
}

impl ScannerFinder {
    pub fn new() -> ScannerFinder {
        ScannerFinder {
            scanners: Arc::new(Mutex::new(vec![])),
        }
    }

    fn scanner_found(&self, name: &str) -> bool {
        let scanners = self.scanners.lock().unwrap();
        for scanner in scanners.iter() {
            if scanner.base_url.contains(name) || scanner.device_name.contains(name) {
                return true;
            }
        }

        return false;
    }

    pub fn find(&mut self, name: Option<&str>) -> Result<Vec<Scanner>, ScannerError> {
        let service_type =
            ServiceType::with_sub_types(&"uscan", &"tcp", vec![]).expect("invalid service type");
        log::info!("Looking for scanners with {service_type:?}");

        let mut browser = MdnsBrowser::new(service_type);
        browser.set_service_discovered_callback(Box::new(Self::on_service_discovered));
        browser.set_context(Box::new(Arc::clone(&self.scanners)));

        let event_loop = browser.browse_services()?;
        let timeout = Duration::from_secs(5);
        let end_time = Instant::now() + timeout;
        while Instant::now() < end_time {
            event_loop.poll(end_time - Instant::now()).unwrap();

            if let Some(name) = name {
                if self.scanner_found(name) {
                    log::info!("Found scanner for name {name}");
                    return Ok(self.scanners.lock().unwrap().clone());
                }
            }
        }

        if let Some(name) = name {
            log::info!("No scanner found for name {name}");
            return Err(ScannerError {
                code: ErrorCode::NoScannerFound,
                message: name.to_string(),
            });
        } else {
            let scanners = self.scanners.lock().unwrap();
            log::info!("Found {} scanners on the network", scanners.len());
            return Ok(scanners.clone());
        };
    }

    fn on_service_discovered(
        result: zeroconf::Result<ServiceDiscovery>,
        context: Option<Arc<dyn Any>>,
    ) {
        let service = match result {
            Ok(service) => service,
            Err(err) => {
                log::info!("Error during scanner discovery (continuing): {err}");
                return;
            }
        };

        log::info!("Service discovered: {service:?}",);
        let scanners = context
            .as_ref()
            .expect("Context was passed to on_service_discovered")
            .downcast_ref::<Arc<Mutex<Vec<Scanner>>>>()
            .expect("context can be downcasted to Arc<Mutex<Vec<Scanner>>>");
        let mut scanners = scanners.lock().unwrap();

        let txt: &TxtRecord = match service.txt() {
            Some(txt) => txt,
            None => {
                log::warn!("Failed to get txt record for {service:?}");
                return;
            }
        };

        let url_root = match txt.get("rs") {
            Some(root) => root,
            None => {
                log::warn!("Service has no resource root (rs): {service:?}");
                return;
            }
        };
        let device_name = match txt.get("ty") {
            Some(name) => name,
            None => {
                log::warn!("Service has no human-readable device name (ty): {service:?}");
                return;
            }
        };

        let scanner = match Scanner::new(&device_name, service.host_name(), &url_root) {
            Ok(scanner) => scanner,
            Err(err) => {
                log::warn!("Failed to initialize scanner {device_name}: {err}");
                return;
            }
        };

        log::info!("{:?}", scanner);
        scanners.push(scanner);
    }
}
