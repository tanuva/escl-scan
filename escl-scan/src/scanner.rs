/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate reqwest;
extern crate serde;
extern crate serde_xml_rs;
extern crate uuid;

use crate::{
    scannererror::{ErrorCode, ScannerError},
    structs::{self},
};
use lopdf::{Bookmark, Document, Object, ObjectId};
use std::{collections::BTreeMap, fmt::Display, fs, path::Path};
use uuid::Uuid;

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
        let response = reqwest::blocking::get(&format!("{}/ScannerCapabilities", base_url))?;
        let response_string = response.text().expect("text is a string");
        log::debug!("> Capabilities: {response_string}");
        let scanner_capabilities: structs::ScannerCapabilities =
            serde_xml_rs::from_str(&response_string)?;
        Ok(scanner_capabilities)
    }

    pub fn new(
        device_name: &str,
        ip_or_host: &str,
        resource_root: &str,
    ) -> Result<Scanner, ScannerError> {
        let base_url = Scanner::make_base_url(ip_or_host, resource_root);
        let capabilities = Self::get_capabilities(&base_url)?;

        Ok(Scanner {
            device_name: device_name.to_string(),
            base_url,
            capabilities,
        })
    }

    pub fn get_status(&self) -> Result<structs::ScannerState, ScannerError> {
        log::info!("Getting scanner status");
        let response = reqwest::blocking::get(&format!("{}/ScannerStatus", self.base_url))?;
        log::debug!("ScannerStatus: {:?}", response);

        let response_string = response.text().expect("text is a string");
        log::debug!("ScannerStatus: {:?}", response_string);

        let scanner_status: structs::ScannerStatus = serde_xml_rs::from_str(&response_string)?;
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
            content_type: "Auto".to_string(),
            input_source: "Platen".to_string(),
            color_mode: "RGB24".to_string(),
            document_format: "image/jpeg".to_string(),
            feed_direction: structs::FeedDirection::ShortEdgeFeed.into(),
            x_resolution: 300,
            y_resolution: 300,
        }
    }

    pub fn scan(
        &self,
        scan_settings: &structs::ScanSettings,
        destination_file: &str,
    ) -> Result<(), ScannerError> {
        let request_body = serde_xml_rs::to_string(scan_settings)?;

        log::info!("Sending scan request with settings: {:?}", scan_settings);
        let client = reqwest::blocking::Client::new();
        let request = client
            .post(format!("{}/ScanJobs", &self.base_url).as_str())
            .body(format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>{}",
                request_body
            ));
        log::debug!("< ScanJobs: {request:#?}\nBody: {request_body:#?}");

        let response = request.send()?;
        log::debug!("> ScanJobs: {response:#?}");

        if response.status().is_client_error() || response.status().is_server_error() {
            return Err(ScannerError {
                code: ErrorCode::NetworkError,
                message: format!(
                    "Status Code: {:?}, Text: {}",
                    response.status(),
                    response.text()?
                ),
            });
        }

        let location = match response.headers().get("location") {
            Some(location) => location.to_str().expect("'location' can be a string"),
            None => {
                return Err(ScannerError {
                    code: ErrorCode::ProtocolError,
                    message: format!(
                        "Failed to get 'location' header from response:\n{response:#?}"
                    ),
                });
            }
        };

        let download_url = format!("{}/NextDocument", location);
        return self.download_scanned_pages(&scan_settings, &download_url, destination_file);
    }

    fn download_scanned_pages(
        &self,
        scan_settings: &structs::ScanSettings,
        download_url: &str,
        destination_file: &str,
    ) -> Result<(), ScannerError> {
        // We need to try downloadng pages until we get a 404 for the printer to
        // consider the scan job done.
        // This is necessary on my Brother MFC-L2710DW to get it to idle state
        // again. It will wait for timeout otherwise, even if we got the scanned
        // page earlier.

        let mut new_page_idx: u16 = 1;
        loop {
            let tmp_page_path = std::env::temp_dir().join(Uuid::new_v4().to_string());
            log::info!(
                "Downloading page {} to {}",
                new_page_idx,
                tmp_page_path.to_str().expect("File path is printable")
            );
            let mut tmp_page_file = fs::File::create(&tmp_page_path)?;

            if let Err(err) = self.download_scanned_page(download_url, &mut tmp_page_file) {
                match err.code {
                    ErrorCode::NoMorePages => {
                        if new_page_idx == 1 {
                            log::error!("Scanner has no pages available for download at all");
                            return Err(err);
                        } else {
                            log::info!("There is no page {new_page_idx}, we're done");
                            return Ok(());
                        }
                    }
                    _ => return Err(err),
                }
            }

            if let Err(err) = self.process_scanned_page(
                scan_settings,
                &tmp_page_path,
                destination_file,
                new_page_idx,
            ) {
                fs::remove_file(tmp_page_path)?;
                return Err(err);
            }

            new_page_idx += 1;
        }
    }

    fn download_scanned_page(
        &self,
        download_url: &str,
        destination_file: &mut fs::File,
    ) -> Result<(), ScannerError> {
        let mut response = reqwest::blocking::get(download_url)?;
        if response.status() == 404 {
            return Err(ScannerError {
                code: ErrorCode::NoMorePages,
                message: String::new(),
            });
        }

        response.copy_to(destination_file)?;
        Ok(())
    }

    fn process_scanned_page(
        &self,
        scan_settings: &structs::ScanSettings,
        tmp_page_path: &Path,
        destination_file: &str,
        page_idx: u16,
    ) -> Result<(), ScannerError> {
        // This could be more elegant if I managed to (de)serialize the format
        // to/from an enum...
        if scan_settings.document_format.contains("pdf") {
            if Path::new(destination_file).exists() {
                log::info!("Appending page to existing document {destination_file}");
                self.merge_scanned_page(Path::new(destination_file), tmp_page_path)?;
            } else {
                log::info!("Storing scanned page as {destination_file}");
                fs::rename(tmp_page_path, &destination_file)?;
            }
        } else {
            let page_file_name = self.make_jpg_file_name(&destination_file, page_idx)?;
            log::info!("Storing scanned page as {page_file_name}");
            fs::rename(tmp_page_path, &page_file_name)?;
        }

        Ok(())
    }

    fn make_jpg_file_name(
        &self,
        destination_file: &str,
        new_page_idx: u16,
    ) -> Result<String, ScannerError> {
        if !Path::new(destination_file).exists() {
            log::info!("Destination file does not exist yet");
            return Ok(destination_file.into());
        }

        // We cannot write to destination_file, try numbering pages
        if let Some(last_dot_pos) = destination_file.rfind('.') {
            let (path_part, ext_part) = destination_file.split_at(last_dot_pos);

            let page_file_name = format!("{path_part}_{new_page_idx}{ext_part}");
            if !Path::new(&page_file_name).exists() {
                log::info!("Created page file name \"{page_file_name}\"");
                return Ok(page_file_name);
            }

            let numbered_path_part = format!("{path_part}_{new_page_idx}");
            for i in 1..u16::MAX {
                let fallback_numbered_file_name = format!("{numbered_path_part}_{i}{ext_part}");
                if !Path::new(&fallback_numbered_file_name).exists() {
                    log::info!("Created fallback file name \"{fallback_numbered_file_name}\"");
                    return Ok(fallback_numbered_file_name);
                }
            }

            return Err(ScannerError {
                code: ErrorCode::FilesystemError,
                message: format!(
                    "Failed to determine an alternative to existing destination file: \"{}\"",
                    destination_file.to_string()
                ),
            });
        }

        Err(ScannerError {
            code: ErrorCode::NoFileExtension,
            message: destination_file.to_string(),
        })
    }

    // Modelled after the lopdf example: https://crates.io/crates/lopdf
    // This must be doable by extending the existing document, too!
    fn merge_scanned_page(&self, output_path: &Path, page_path: &Path) -> Result<(), ScannerError> {
        assert!(output_path.is_file());
        assert!(page_path.is_file());

        let documents = vec![Document::load(output_path)?, Document::load(page_path)?];

        // Define a starting max_id (will be used as start index for object_ids)
        let mut max_id = 1;
        let mut pagenum = 1;
        // Collect all Documents Objects grouped by a map
        let mut documents_pages = BTreeMap::new();
        let mut documents_objects = BTreeMap::new();
        let mut document = Document::with_version("1.5");

        for mut doc in documents {
            let mut first = false;
            doc.renumber_objects_with(max_id);

            max_id = doc.max_id + 1;

            documents_pages.extend(
                doc.get_pages()
                    .into_iter()
                    .map(|(_, object_id)| {
                        if !first {
                            let bookmark = Bookmark::new(
                                String::from(format!("Page_{}", pagenum)),
                                [0.0, 0.0, 1.0],
                                0,
                                object_id,
                            );
                            document.add_bookmark(bookmark, None);
                            first = true;
                            pagenum += 1;
                        }

                        (object_id, doc.get_object(object_id).unwrap().to_owned())
                    })
                    .collect::<BTreeMap<ObjectId, Object>>(),
            );
            documents_objects.extend(doc.objects);
        }

        // Catalog and Pages are mandatory
        let mut catalog_object: Option<(ObjectId, Object)> = None;
        let mut pages_object: Option<(ObjectId, Object)> = None;

        // Process all objects except "Page" type
        for (object_id, object) in documents_objects.iter() {
            // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
            // All other objects should be collected and inserted into the main Document
            match object.type_name().unwrap_or("") {
                "Catalog" => {
                    // Collect a first "Catalog" object and use it for the future "Pages"
                    catalog_object = Some((
                        if let Some((id, _)) = catalog_object {
                            id
                        } else {
                            *object_id
                        },
                        object.clone(),
                    ));
                }
                "Pages" => {
                    // Collect and update a first "Pages" object and use it for the future "Catalog"
                    // We have also to merge all dictionaries of the old and the new "Pages" object
                    if let Ok(dictionary) = object.as_dict() {
                        let mut dictionary = dictionary.clone();
                        if let Some((_, ref object)) = pages_object {
                            if let Ok(old_dictionary) = object.as_dict() {
                                dictionary.extend(old_dictionary);
                            }
                        }

                        pages_object = Some((
                            if let Some((id, _)) = pages_object {
                                id
                            } else {
                                *object_id
                            },
                            Object::Dictionary(dictionary),
                        ));
                    }
                }
                "Page" => {}     // Ignored, processed later and separately
                "Outlines" => {} // Ignored, not supported yet
                "Outline" => {}  // Ignored, not supported yet
                _ => {
                    document.objects.insert(*object_id, object.clone());
                }
            }
        }

        // If no "Pages" object found abort
        if pages_object.is_none() {
            println!("Pages root not found.");
            return Ok(());
        }

        // Iterate over all "Page" objects and collect into the parent "Pages" created before
        for (object_id, object) in documents_pages.iter() {
            if let Ok(dictionary) = object.as_dict() {
                let mut dictionary = dictionary.clone();
                dictionary.set("Parent", pages_object.as_ref().unwrap().0);
                document
                    .objects
                    .insert(*object_id, Object::Dictionary(dictionary));
            }
        }

        // If no "Catalog" found abort
        if catalog_object.is_none() {
            println!("Catalog root not found.");
            return Ok(());
        }

        let catalog_object = catalog_object.unwrap();
        let pages_object = pages_object.unwrap();

        // Build a new "Pages" with updated fields
        if let Ok(dictionary) = pages_object.1.as_dict() {
            let mut dictionary = dictionary.clone();

            // Set new pages count
            dictionary.set("Count", documents_pages.len() as u32);

            // Set new "Kids" list (collected from documents pages) for "Pages"
            dictionary.set(
                "Kids",
                documents_pages
                    .into_iter()
                    .map(|(object_id, _)| Object::Reference(object_id))
                    .collect::<Vec<_>>(),
            );

            document
                .objects
                .insert(pages_object.0, Object::Dictionary(dictionary));
        }

        // Build a new "Catalog" with updated fields
        if let Ok(dictionary) = catalog_object.1.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Pages", pages_object.0);
            dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs
            document
                .objects
                .insert(catalog_object.0, Object::Dictionary(dictionary));
        }

        document.trailer.set("Root", catalog_object.0);

        // Update the max internal ID as wasn't updated before due to direct objects insertion
        document.max_id = document.objects.len() as u32;

        // Reorder all new Document objects
        document.renumber_objects();

        //Set any Bookmarks to the First child if they are not set to a page
        document.adjust_zero_pages();

        //Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
        if let Some(n) = document.build_outline() {
            if let Ok(x) = document.get_object_mut(catalog_object.0) {
                if let Object::Dictionary(ref mut dict) = x {
                    dict.set("Outlines", Object::Reference(n));
                }
            }
        }

        document.compress();
        if let Err(err) = document.save(output_path) {
            log::error!("Failed to save merged pdf document: {err:?}");
            return Err(err.into());
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
