/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate serde;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize)]
pub struct Platen {
    #[serde(rename = "PlatenInputCaps", default)]
    pub platen_input_caps: PlatenInputCaps,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct PlatenInputCaps {
    #[serde(rename = "MinWidth", default)]
    pub min_width: u16,
    #[serde(rename = "MaxWidth", default)]
    pub max_width: u16,
    #[serde(rename = "MinHeight", default)]
    pub min_height: u16,
    #[serde(rename = "MaxHeight", default)]
    pub max_height: u16,
    #[serde(rename = "MaxScanRegions", default)]
    pub max_scan_regions: u16,
    #[serde(rename = "SettingProfiles")]
    pub setting_profiles: SettingProfiles,
    // TODO: Make SupportedIntents
    #[serde(rename = "MaxOpticalXResolution", default)]
    pub max_optical_xresolution: u16,
    #[serde(rename = "MaxOpticalYResolution", default)]
    pub max_optical_yresolution: u16,
    #[serde(rename = "RiskyLeftMargin", default)]
    pub risky_left_margin: u16,
    #[serde(rename = "RiskyRightMargin", default)]
    pub risky_right_margin: u16,
    #[serde(rename = "RiskyTopMargin", default)]
    pub risky_top_margin: u16,
    #[serde(rename = "RiskyBottomMargin", default)]
    pub risky_bottom_margin: u16,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct SettingProfiles {
    #[serde(rename = "$value")]
    pub entries: Vec<SettingProfile>,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct SettingProfile {
    #[serde(rename = "ColorModes")]
    pub color_modes: ColorModes,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct ColorModes {
    #[serde(rename = "$value")]
    pub entries: Vec<ColorMode>,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct ColorMode {
    #[serde(rename = "$value")]
    pub mode_name: String,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct CompressionFactorSupport {
    #[serde(rename = "Min", default)]
    pub min: i8,
    #[serde(rename = "Max", default)]
    pub max: i8,
    #[serde(rename = "Normal", default)]
    pub normal: i8,
    #[serde(rename = "Step", default)]
    pub step: i8,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct SupportedMediaTypes {
    #[serde(rename = "MediaType", default)]
    pub media_types: Vec<String>,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct SharpenSupport {
    #[serde(rename = "Min", default)]
    pub min: i8,
    #[serde(rename = "Max", default)]
    pub max: i8,
    #[serde(rename = "Normal", default)]
    pub normal: i8,
    #[serde(rename = "Step", default)]
    pub step: i8,
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct ScannerCapabilities {
    #[serde(rename = "Version", default)]
    pub version: String,
    #[serde(rename = "MakeAndModel", default)]
    pub make_and_model: String,
    #[serde(rename = "SerialNumber", default)]
    pub serial_number: String,
    #[serde(rename = "UUID", default)]
    pub uuid: String,
    #[serde(rename = "AdminURI", default)]
    pub admin_uri: String,
    #[serde(rename = "IconURI", default)]
    pub icon_uri: String,
    #[serde(rename = "Platen", default)]
    pub platen: Platen,
    #[serde(rename = "CompressionFactorSupport", default)]
    pub compression_factor_support: CompressionFactorSupport,
    #[serde(rename = "SupportedMediaTypes", default)]
    pub supported_media_types: SupportedMediaTypes,
    #[serde(rename = "SharpenSupport", default)]
    pub sharpen_support: SharpenSupport,
}

#[derive(Default, Debug, Deserialize, PartialEq)]
#[serde(rename = "State")]
pub enum ScannerState {
    Idle,
    Processing,
    Testing,
    Stopped,
    #[default]
    Down,
}

impl Display for ScannerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", *self)
    }
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename = "scan:ScannerStatus")]
pub struct ScannerStatus {
    #[serde(rename = "State")]
    pub state: ScannerState,
    #[serde(rename = "AdfState")]
    pub adf_state: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "scan:ScanRegion")]
pub struct ScanRegion {
    #[serde(rename = "pwg:XOffset")]
    pub x_offset: i16,
    #[serde(rename = "pwg:YOffset")]
    pub y_offset: i16,
    #[serde(rename = "pwg:Width")]
    pub width: u16,
    #[serde(rename = "pwg:Height")]
    pub height: u16,
    #[serde(rename = "pwg:ContentRegionUnits")]
    pub content_region_units: String,
}

impl ScanRegion {
    fn from_mm(width: usize, height: usize) -> ScanRegion {
        let mm_to_300th_inch_factor: f32 = 0.03937 * 300.0;
        ScanRegion {
            x_offset: 0,
            y_offset: 0,
            width: (width as f32 * mm_to_300th_inch_factor) as u16,
            height: (height as f32 * mm_to_300th_inch_factor) as u16,
            content_region_units: "escl:ThreeHundredthsOfInches".to_string(),
        }
    }

    pub fn a4_portrait() -> ScanRegion {
        Self::from_mm(210, 297)
    }

    pub fn a5_portrait() -> ScanRegion {
        Self::from_mm(148, 210)
    }

    pub fn a5_landscape() -> ScanRegion {
        Self::from_mm(210, 148)
    }

    pub fn us_letter_portrait() -> ScanRegion {
        // Slightly rounded values...
        Self::from_mm(216, 279)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "scan:ScanSettings")]
pub struct ScanSettings {
    #[serde(rename = "pwg:Version")]
    pub version: String,
    #[serde(rename = "pwg:ContentType")]
    pub content_type: String,
    #[serde(rename = "pwg:InputSource")]
    pub input_source: String,
    #[serde(rename = "pwg:ScanRegions")]
    pub scan_regions: ScanRegion,
    #[serde(rename = "scan:ColorMode")]
    pub color_mode: String,
    #[serde(rename = "scan:DocumentFormatExt")]
    pub document_format: String,
    #[serde(rename = "scan:FeedDirection")]
    pub feed_direction: String,
    #[serde(rename = "scan:XResolution")]
    pub x_resolution: i16,
    #[serde(rename = "scan:YResolution")]
    pub y_resolution: i16,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "$value")]
pub enum FeedDirection {
    LongEdgeFeed,
    #[default]
    ShortEdgeFeed,
}

impl From<FeedDirection> for String {
    fn from(value: FeedDirection) -> Self {
        match value {
            FeedDirection::LongEdgeFeed => "LongEdgeFeed".to_string(),
            FeedDirection::ShortEdgeFeed => "ShortEdgeFeed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::structs::*;
    use std::io::Read;

    #[test]
    fn scanner_capabilities() {
        let xml_file_result =
            std::fs::File::open("../reference/Brother_MFC-2710DW_Capabilities.xml");
        assert!(xml_file_result.is_ok());
        let mut xml = String::new();
        assert!(xml_file_result
            .ok()
            .unwrap()
            .read_to_string(&mut xml)
            .is_ok());

        let result = serde_xml_rs::from_str::<ScannerCapabilities>(&xml);
        if let Err(err) = result {
            println!("{err}");
            assert!(false);
            return;
        }

        assert!(result.is_ok());
        let caps = result.ok().unwrap();
        let setting_profiles = caps.platen.platen_input_caps.setting_profiles.entries;
        assert!(setting_profiles.len() == 1);
        let platen_profile = setting_profiles.first().unwrap();
        let color_modes = &platen_profile.color_modes.entries;
        assert!(color_modes.len() == 3);
        assert!(color_modes[0].mode_name == "BlackAndWhite1");
        assert!(color_modes[1].mode_name == "Grayscale8");
        assert!(color_modes[2].mode_name == "RGB24");
    }

    #[test]
    fn scanner_status() {
        let xml_file_result =
            std::fs::File::open("../reference/Brother_MFC-2710DW_ScannerStatus.xml");
        assert!(xml_file_result.is_ok());
        let mut xml = String::new();
        assert!(xml_file_result
            .ok()
            .unwrap()
            .read_to_string(&mut xml)
            .is_ok());

        let result = serde_xml_rs::from_str::<ScannerStatus>(&xml);
        assert!(result.is_ok());
        let status = result.ok().unwrap();
        assert!(status.state == ScannerState::Idle);
        // TODO adf_state
    }
}
