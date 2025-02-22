/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use core::fmt;

#[derive(Debug)]
pub enum ErrorCode {
    FilesystemError,
    NetworkError,
    NoFileExtension,
    NoMorePages,
    NoScannerFound,
    PdfError,
    ProtocolError,
    ScannerNotReady,
}

#[derive(Debug)]
pub struct ScannerError {
    pub code: ErrorCode,
    pub message: String,
}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self.code {
            ErrorCode::FilesystemError => format!("File System Error: {}", self.message),
            ErrorCode::NetworkError => format!("Network Error: {}", self.message),
            ErrorCode::NoFileExtension => format!(
                "Specified output file does not have a file extension: {}",
                self.message
            ),
            ErrorCode::NoMorePages => {
                "There are no more scanned pages available for download".to_string()
            }
            ErrorCode::NoScannerFound => {
                format!("No scanner found where name contains \"{}\"", self.message)
            }
            ErrorCode::PdfError => format!("PDF processing error: {}", self.message),
            ErrorCode::ProtocolError => format!("eSCL Protocol Error: {}", self.message),
            ErrorCode::ScannerNotReady => "The scanner is not ready to scan".to_string(),
        };

        write!(f, "{}", msg)
    }
}

impl From<lopdf::Error> for ScannerError {
    fn from(error: lopdf::Error) -> Self {
        ScannerError {
            code: ErrorCode::PdfError,
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for ScannerError {
    fn from(error: reqwest::Error) -> Self {
        ScannerError {
            code: ErrorCode::NetworkError,
            message: error.to_string(),
        }
    }
}

impl From<serde_xml_rs::Error> for ScannerError {
    fn from(error: serde_xml_rs::Error) -> Self {
        ScannerError {
            code: ErrorCode::ProtocolError,
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for ScannerError {
    fn from(error: std::io::Error) -> Self {
        ScannerError {
            code: ErrorCode::FilesystemError,
            message: error.to_string(),
        }
    }
}

impl From<zeroconf::error::Error> for ScannerError {
    fn from(error: zeroconf::error::Error) -> Self {
        ScannerError {
            code: ErrorCode::NetworkError,
            message: error.to_string(),
        }
    }
}
