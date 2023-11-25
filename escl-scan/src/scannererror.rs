use core::fmt;

#[derive(Debug)]
pub enum ErrorCode {
    FilesystemError,
    NetworkError,
    NoScannerFound,
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
            ErrorCode::NoScannerFound => {
                format!("No scanner found where name contains {}", self.message)
            }
            ErrorCode::ProtocolError => format!("eSCL Protocol Error: {}", self.message),
            ErrorCode::ScannerNotReady => "The scanner is not ready to scan".to_string(),
        };

        write!(f, "{}", msg)
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
