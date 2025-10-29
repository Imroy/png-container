/*
  png-container
  Copyright (C) 2023 Ian Tester

  This program is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

/*! PNG chunks
 */

use std::io::{Read, Seek, SeekFrom};
use std::slice::Iter;
use std::str;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use flate2::{
    Compression,
    bufread::{ZlibDecoder, ZlibEncoder},
};
use uom::si::{
    f64::{LinearNumberDensity, Luminance, Time},
    linear_number_density::per_meter,
    luminance::candela_per_square_meter,
};

use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Enum of PNG chunk types and the data they hold
#[derive(Clone, Debug)]
pub enum PngChunkData {
    /// Empty type
    None,

    // Critical chunks

    /// Image header
    Ihdr {
        /// Width of image in pixels
        width: u32,

        /// Height of image in pixels
        height: u32,

        /// Number of bits per sample
        bit_depth: u8,

        /// Colour type
        colour_type: PngColourType,

        /// Compression method
        compression_method: PngCompressionMethod,

        /// Filter method
        filter_method: PngFilterMethod,

        /// Interlace method
        interlace_method: PngInterlaceMethod,
    },

    /// Palette
    Plte(Box<Vec<PngPaletteEntry>>),

    /// Image data
    Idat(Box<Vec<u8>>),

    /// Image end
    Iend,

    // Transparency information
    /// Transparency
    Trns { data: PngTrnsType },

    // Colour space information
    /// Primary chromaticities and white point
    Chrm(Box<Chrm>),

    /// Image gamma
    ///
    /// Value is scaled by 100000
    Gama { gamma: u32 },

    /// Embedded ICC profile
    Iccp(Box<Iccp>),

    /// Significant bits
    Sbit { bits: PngSbitType },

    /// Standard RGB colour space
    Srgb {
        rendering_intent: PngRenderingIntent,
    },

    /// Coding-independent code points for video signal type identification
    Cicp {
        colour_primaries: ColourPrimaries,
        transfer_function: TransferFunction,
        matrix_coeffs: MatrixCoefficients,
        video_full_range: bool,
    },

    /// Mastering Display Color Volume
    Mdcv(Box<Mdcv>),

    /// Content Light Level Information
    Clli(Box<Clli>),

    // Textual information
    /// Textual data
    Text(Box<Text>),

    /// Compressed textual data
    Ztxt(Box<Ztxt>),

    /// International textual data
    Itxt(Box<Itxt>),

    // Miscellaneous information
    /// Background colour
    Bkgd { data: PngBkgdType },

    /// Image histogram
    Hist(Box<Vec<u16>>),

    /// Physical pixel dimensions
    Phys {
        x_pixels_per_unit: u32,
        y_pixels_per_unit: u32,
        unit: PngUnitType,
    },

    /// Suggested palette
    Splt(Box<Splt>),

    /// Exchangeable Image File (Exif) Profile
    Exif(Box<Vec<u8>>),

    // Time stamp information
    /// Image last-modification time
    Time {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    },

    /// Animation control
    Actl { num_frames: u32, num_plays: u32 },

    /// Frame control
    Fctl(Box<Fctl>),

    /// Frame data
    Fdat(Box<Fdat>),

    // Extensions
    /// Image offset
    Offs { x: u32, y: u32, unit: PngUnitType },

    /// Calibration of pixel values
    Pcal(Box<Pcal>),

    /// Physical scale of image subject
    Scal(Box<Scal>),

    /// GIF Graphic Control Extension
    Gifg {
        disposal_method: GifDisposalMethod,
        user_input: bool,
        delay_time: u16,
    },

    /// GIF Application Extension
    Gifx(Box<Gifx>),

    /// Indicator of Stereo Image
    Ster { mode: u8 },

    // JNG chunks
    /// JNG header
    Jhdr {
        /// Width of image in pixels
        width: u32,

        /// Height of image in pixels
        height: u32,

        /// Colour type
        colour_type: JngColourType,

        /// Image sample depth
        image_sample_depth: JngImageSampleDepth,

        /// Image compression method
        image_compression_method: JngCompressionType,

        /// Image interlace method
        image_interlace_method: JngInterlaceMethod,

        /// Alpha sample depth
        alpha_sample_depth: JngAlphaSampleDepth,

        /// Alpha compression method
        alpha_compression_method: JngCompressionType,

        /// Alpha channel filter method
        alpha_filter_method: PngFilterMethod,

        /// Alpha interlace method
        alpha_interlace_method: JngInterlaceMethod,
    },

    /// JNG image data
    Jdat(Box<Vec<u8>>),

    /// JNG alpha data
    Jdaa(Box<Vec<u8>>),

    /// JNG image separator
    Jsep,
}

impl PngChunkData {
    /// Return an iterator into the data of IDAT/fdAT/JDAT/JDAA chunks
    pub fn dat_data_iter(&self) -> Option<Iter<'_, u8>> {
        match self {
            PngChunkData::Idat(data) => Some(data.iter()),

            PngChunkData::Fdat(fdat) => Some(fdat.frame_data.iter()),

            PngChunkData::Jdat(data) => Some(data.iter()),

            PngChunkData::Jdaa(data) => Some(data.iter()),

            _ => None,
        }
    }

    /// Scaled white coordinates of the cHRM chunk
    pub fn chrm_white_coords(&self) -> Option<(f64, f64)> {
        if let PngChunkData::Chrm(chrm) = self {
            return Some(chrm.white_coords());
        }

        None
    }

    /// Scaled red coordinates of the cHRM chunk
    pub fn chrm_red_coords(&self) -> Option<(f64, f64)> {
        if let PngChunkData::Chrm(chrm) = self {
            return Some(chrm.red_coords());
        }

        None
    }

    /// Scaled green coordinates of the cHRM chunk
    pub fn chrm_green_coords(&self) -> Option<(f64, f64)> {
        if let PngChunkData::Chrm(chrm) = self {
            return Some(chrm.green_coords());
        }

        None
    }

    /// Scaled blue coordinates of the cHRM chunk
    pub fn chrm_blue_coords(&self) -> Option<(f64, f64)> {
        if let PngChunkData::Chrm(chrm) = self {
            return Some(chrm.blue_coords());
        }

        None
    }

    /// Scaled gamma value of a gAMA chunk
    pub fn gama_gamma(&self) -> Option<f64> {
        if let PngChunkData::Gama { gamma } = self {
            return Some(*gamma as f64 / 100000.0);
        }

        None
    }

    /// Decompress the compressed profile in a iCCP chunk
    pub fn iccp_profile(&self) -> Option<Vec<u8>> {
        if let PngChunkData::Iccp(iccp) = self {
            iccp.profile()
        } else {
            None
        }
    }

    /// Decompress the compressed string in a zTXt chunk
    pub fn ztxt_string(&self) -> Option<String> {
        if let PngChunkData::Ztxt(ztxt) = self {
            return ztxt.string();
        }

        None
    }

    /// Decompress the compressed string in an iTXt chunk
    pub fn itxt_string(&self) -> Option<String> {
        if let PngChunkData::Itxt(itxt) = self {
            return itxt.string();
        }

        None
    }

    /// Convert the units in a pHYs chunk to a UoM type
    pub fn phys_res(&self) -> Option<(LinearNumberDensity, LinearNumberDensity)> {
        if let PngChunkData::Phys {
            x_pixels_per_unit,
            y_pixels_per_unit,
            unit,
        } = self
        {
            return match unit {
                PngUnitType::Unknown => None,

                PngUnitType::Metre => Some((
                    LinearNumberDensity::new::<per_meter>(*x_pixels_per_unit as f64),
                    LinearNumberDensity::new::<per_meter>(*y_pixels_per_unit as f64),
                )),
            };
        }

        None
    }

    /// Convert the timestamp in a tIME chunk to a chrono DateTime object
    pub fn time(&self) -> Option<DateTime<Utc>> {
        if let PngChunkData::Time {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } = self
        {
            return Some(DateTime::from_naive_utc_and_offset(
                NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(*year as i32, *month as u32, *day as u32)?,
                    NaiveTime::from_hms_opt(*hour as u32, *minute as u32, *second as u32)?,
                ),
                Utc,
            ));
        }

        None
    }

    /// Calculate delay from fcTL chunk in seconds
    pub fn fctl_delay(&self) -> Option<Time> {
        if let PngChunkData::Fctl(fctl) = self {
            return Some(fctl.delay());
        }

        None
    }
}

/// Primary chromaticities and white point
///
/// Values are scaled by 100000
#[derive(Clone, Copy, Debug, Default)]
pub struct Chrm {
    pub white_x: u32,
    pub white_y: u32,
    pub red_x: u32,
    pub red_y: u32,
    pub green_x: u32,
    pub green_y: u32,
    pub blue_x: u32,
    pub blue_y: u32,
}

impl Chrm {
    /// Constructor
    pub fn new(white: (f64, f64), red: (f64, f64), green: (f64, f64), blue: (f64, f64)) -> Self {
        Self {
            white_x: (white.0 * 100000.0) as u32,
            white_y: (white.1 * 100000.0) as u32,
            red_x: (red.0 * 100000.0) as u32,
            red_y: (red.1 * 100000.0) as u32,
            green_x: (green.0 * 100000.0) as u32,
            green_y: (green.1 * 100000.0) as u32,
            blue_x: (blue.0 * 100000.0) as u32,
            blue_y: (blue.1 * 100000.0) as u32,
        }
    }

    /// Set the white coordinates
    pub fn set_white_coords(&mut self, white: (f64, f64)) {
        self.white_x = (white.0 * 100000.0) as u32;
        self.white_y = (white.1 * 100000.0) as u32;
    }

    /// Scaled white coordinates of the cHRM chunk
    pub fn white_coords(&self) -> (f64, f64) {
        (
            self.white_x as f64 / 100000.0,
            self.white_y as f64 / 100000.0,
        )
    }

    /// Set the red coordinates
    pub fn set_red_coords(&mut self, red: (f64, f64)) {
        self.red_x = (red.0 * 100000.0) as u32;
        self.red_y = (red.1 * 100000.0) as u32;
    }

    /// Scaled red coordinates of the cHRM chunk
    pub fn red_coords(&self) -> (f64, f64) {
        (self.red_x as f64 / 100000.0, self.red_y as f64 / 100000.0)
    }

    /// Set the green coordinates
    pub fn set_green_coords(&mut self, green: (f64, f64)) {
        self.green_x = (green.0 * 100000.0) as u32;
        self.green_y = (green.1 * 100000.0) as u32;
    }

    /// Scaled green coordinates of the cHRM chunk
    pub fn green_coords(&self) -> (f64, f64) {
        (
            self.green_x as f64 / 100000.0,
            self.green_y as f64 / 100000.0,
        )
    }

    /// Set the blue coordinates
    pub fn set_blue_coords(&mut self, blue: (f64, f64)) {
        self.blue_x = (blue.0 * 100000.0) as u32;
        self.blue_y = (blue.1 * 100000.0) as u32;
    }

    /// Scaled blue coordinates of the cHRM chunk
    pub fn blue_coords(&self) -> (f64, f64) {
        (self.blue_x as f64 / 100000.0, self.blue_y as f64 / 100000.0)
    }
}

/// Mastering Display Color Volume
#[derive(Clone, Copy, Debug, Default)]
pub struct Mdcv {
    pub red_x: u16,
    pub red_y: u16,
    pub green_x: u16,
    pub green_y: u16,
    pub blue_x: u16,
    pub blue_y: u16,
    pub white_x: u16,
    pub white_y: u16,
    pub max_lum: u32,
    pub min_lum: u32,
}

impl Mdcv {
    /// Constructor
    pub fn new(
        red: (f64, f64),
        green: (f64, f64),
        blue: (f64, f64),
        white: (f64, f64),
        max_lum: Luminance,
        min_lum: Luminance,
    ) -> Self {
        Self {
            red_x: (red.0 * 50000.0) as u16,
            red_y: (red.1 * 50000.0) as u16,
            green_x: (green.0 * 50000.0) as u16,
            green_y: (green.1 * 50000.0) as u16,
            blue_x: (blue.0 * 50000.0) as u16,
            blue_y: (blue.1 * 50000.0) as u16,
            white_x: (white.0 * 50000.0) as u16,
            white_y: (white.1 * 50000.0) as u16,
            max_lum: (max_lum.get::<candela_per_square_meter>() * 10000.0) as u32,
            min_lum: (min_lum.get::<candela_per_square_meter>() * 10000.0) as u32,
        }
    }

    /// Set the red coordinates
    pub fn set_red_coords(&mut self, red: (f64, f64)) {
        self.red_x = (red.0 * 50000.0) as u16;
        self.red_y = (red.1 * 50000.0) as u16;
    }

    /// Scaled red coordinates of the mDCV chunk
    pub fn red_coords(&self) -> (f64, f64) {
        (self.red_x as f64 / 50000.0, self.red_y as f64 / 50000.0)
    }

    /// Set the green coordinates
    pub fn set_green_coords(&mut self, green: (f64, f64)) {
        self.green_x = (green.0 * 50000.0) as u16;
        self.green_y = (green.1 * 50000.0) as u16;
    }

    /// Scaled green coordinates of the mDCV chunk
    pub fn green_coords(&self) -> (f64, f64) {
        (self.green_x as f64 / 50000.0, self.green_y as f64 / 50000.0)
    }

    /// Set the blue coordinates
    pub fn set_blue_coords(&mut self, blue: (f64, f64)) {
        self.blue_x = (blue.0 * 50000.0) as u16;
        self.blue_y = (blue.1 * 50000.0) as u16;
    }

    /// Scaled blue coordinates of the mDCV chunk
    pub fn blue_coords(&self) -> (f64, f64) {
        (self.blue_x as f64 / 50000.0, self.blue_y as f64 / 50000.0)
    }

    /// Set the white coordinates
    pub fn set_white_coords(&mut self, white: (f64, f64)) {
        self.white_x = (white.0 * 50000.0) as u16;
        self.white_y = (white.1 * 50000.0) as u16;
    }

    /// Scaled white coordinates of the mDCV chunk
    pub fn white_coords(&self) -> (f64, f64) {
        (self.white_x as f64 / 50000.0, self.white_y as f64 / 50000.0)
    }

    /// Set the maximum luminance of the mDCV chunk
    pub fn set_max_lum(&mut self, max_lum: Luminance) {
        self.max_lum = (max_lum.get::<candela_per_square_meter>() * 10000.0) as u32;
    }

    /// Scaled mastering display maximum luminance of the mDCV chunk
    pub fn max_lum(&self) -> Luminance {
        Luminance::new::<candela_per_square_meter>(self.max_lum as f64 / 10000.0)
    }

    /// Set the minimum luminance of the mDCV chunk
    pub fn set_min_lum(&mut self, min_lum: Luminance) {
        self.min_lum = (min_lum.get::<candela_per_square_meter>() * 10000.0) as u32;
    }

    /// Scaled mastering display minimum luminance of the mDCV chunk
    pub fn min_lum(&self) -> Luminance {
        Luminance::new::<candela_per_square_meter>(self.min_lum as f64 / 10000.0)
    }
}

/// Content Light Level Information
#[derive(Clone, Copy, Debug, Default)]
pub struct Clli {
    /// Maximum Content Light Level
    pub max_cll: u32,

    /// Maximum Frame-Average Light Level
    pub max_fall: u32,
}

impl Clli {
    /// Constructor
    pub fn new(max_cll: Luminance, max_fall: Luminance) -> Self {
        Self {
            max_cll: (max_cll.get::<candela_per_square_meter>() * 10000.0) as u32,
            max_fall: (max_fall.get::<candela_per_square_meter>() * 10000.0) as u32,
        }
    }

    /// Set Maximum Content Light Level
    pub fn set_max_cll(&mut self, max_cll: Luminance) {
        self.max_cll = (max_cll.get::<candela_per_square_meter>() * 10000.0) as u32;
    }

    /// Scaled maximum content light level
    pub fn max_cll(&self) -> Luminance {
        Luminance::new::<candela_per_square_meter>(self.max_cll as f64 / 10000.0)
    }

    /// Set Maximum Frame-Average Light Level
    pub fn set_max_fall(&mut self, max_fall: Luminance) {
        self.max_fall = (max_fall.get::<candela_per_square_meter>() * 10000.0) as u32;
    }

    /// Scaled maximum frame-average Light Level
    pub fn max_fall(&self) -> Luminance {
        Luminance::new::<candela_per_square_meter>(self.max_fall as f64 / 10000.0)
    }
}

/// Embedded ICC profile
#[derive(Clone, Debug, Default)]
pub struct Iccp {
    pub name: String,
    pub compression_method: PngCompressionMethod,
    pub compressed_profile: Vec<u8>,
}

impl Iccp {
    /// Constructor
    pub fn new(name: &str, compression_method: PngCompressionMethod, profile: &[u8]) -> Self {
        let mut compressed_profile = Vec::new();
        if compression_method == PngCompressionMethod::Zlib {
            let mut encoder = ZlibEncoder::new(profile, Compression::best());
            let _ = encoder.read_to_end(&mut compressed_profile);
        }

        Self {
            name: name.to_string(),
            compression_method,
            compressed_profile,
        }
    }

    /// Set profile
    pub fn set_profile(&mut self, compression_method: PngCompressionMethod, profile: &[u8]) {
        let mut compressed_profile = Vec::new();
        if compression_method == PngCompressionMethod::Zlib {
            let mut encoder = ZlibEncoder::new(profile, Compression::best());
            let _ = encoder.read_to_end(&mut compressed_profile);
        }

        self.compression_method = compression_method;
        self.compressed_profile = compressed_profile;
    }

    /// Uncompressed profile
    pub fn profile(&self) -> Option<Vec<u8>> {
        if self.compression_method == PngCompressionMethod::Zlib {
            let mut decoder = ZlibDecoder::new(self.compressed_profile.as_slice());
            let mut out = Vec::new();
            if decoder.read_to_end(&mut out).is_ok() {
                return Some(out);
            }
        }

        None
    }
}

/// Textual data
#[derive(Clone, Debug)]
pub struct Text {
    pub keyword: String,
    pub string: String,
}

/// Compressed textual data
#[derive(Clone, Debug, Default)]
pub struct Ztxt {
    pub keyword: String,
    pub compression_method: PngCompressionMethod,
    pub compressed_string: Vec<u8>,
}

impl Ztxt {
    /// Constructor
    pub fn new(keyword: &str, compression_method: PngCompressionMethod, string: &str) -> Self {
        let mut compressed_string = Vec::new();
        if compression_method == PngCompressionMethod::Zlib {
            let mut encoder = ZlibEncoder::new(string.as_bytes(), Compression::best());
            let _ = encoder.read_to_end(&mut compressed_string);
        }

        Self {
            keyword: keyword.to_string(),
            compression_method,
            compressed_string,
        }
    }

    /// Decompress the compressed string in a zTXt chunk
    pub fn string(&self) -> Option<String> {
        if self.compression_method == PngCompressionMethod::Zlib {
            let mut decoder = ZlibDecoder::new(self.compressed_string.as_slice());
            let mut out = Vec::new();
            if decoder.read_to_end(&mut out).is_ok() {
                return Some(out.iter().map(|b| *b as char).collect());
            }
        }

        None
    }
}

/// International textual data
#[derive(Clone, Debug, Default)]
pub struct Itxt {
    pub keyword: String,
    pub compressed: bool,
    pub compression_method: PngCompressionMethod,
    pub language: String,
    pub translated_keyword: String,
    pub compressed_string: Vec<u8>,
}

impl Itxt {
    /// Constructor
    pub fn new(
        keyword: &str,
        compression_method: Option<PngCompressionMethod>,
        language: &str,
        translated_keyword: &str,
        string: &str,
    ) -> Self {
        let mut compressed_string = Vec::new();
        if compression_method == Some(PngCompressionMethod::Zlib) {
            let mut encoder = ZlibEncoder::new(string.as_bytes(), Compression::best());
            let _ = encoder.read_to_end(&mut compressed_string);
        } else {
            compressed_string.extend(string.bytes());
        }

        Self {
            keyword: keyword.to_string(),
            compressed: compression_method.is_some(),
            compression_method: compression_method.unwrap_or_default(),
            language: language.to_string(),
            translated_keyword: translated_keyword.to_string(),
            compressed_string,
        }
    }

    /// Decompress the compressed string in an iTXt chunk
    pub fn string(&self) -> Option<String> {
        if self.compressed {
            if self.compression_method == PngCompressionMethod::Zlib {
                let mut decoder = ZlibDecoder::new(self.compressed_string.as_slice());
                let mut out = String::new();
                if decoder.read_to_string(&mut out).is_ok() {
                    return Some(out);
                }
            }

            return None;
        }

        String::from_utf8(self.compressed_string.to_vec()).ok()
    }
}

/// Suggested palette
#[derive(Clone, Debug, Default)]
pub struct Splt {
    pub name: String,
    pub depth: u8,
    pub palette: Vec<PngSuggestedPaletteEntry>,
}

/// Calibration of pixel values
#[derive(Clone, Debug, Default)]
pub struct Pcal {
    pub name: String,
    pub original_zero: u32,
    pub original_max: u32,
    pub equation_type: u8,
    pub unit_name: String,
    pub parameters: Vec<String>,
}

/// Physical scale of image subject
#[derive(Clone, Debug)]
pub struct Scal {
    pub unit: PngUnitType,
    pub pixel_width: String,
    pub pixel_height: String,
}

/// GIF Application Extension
#[derive(Clone, Debug)]
pub struct Gifx {
    pub app_id: String,
    pub app_auth: [u8; 3],
    pub app_data: Vec<u8>,
}

/// Frame control
#[derive(Clone, Debug)]
pub struct Fctl {
    pub sequence_number: u32,
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub delay_num: u16,
    pub delay_den: u16,
    pub dispose_op: ApngDisposalOperator,
    pub blend_op: ApngBlendOperator,
}

impl Fctl {
    /// Calculate delay from fcTL chunk in seconds
    pub fn delay(&self) -> Time {
        Time::new::<uom::si::time::second>(self.delay_num as f64 / self.delay_den as f64)
    }
}

/// Frame data
#[derive(Clone, Debug)]
pub struct Fdat {
    pub sequence_number: u32,
    pub frame_data: Vec<u8>,
}

/// Reference to a chunk in a PNG file
#[derive(Copy, Clone, Debug, Default)]
pub struct PngChunkRef {
    /// The position in the stream/file for this chunk
    pub position: u64,

    /// Length of this chunk
    pub length: u32,

    /// Chunk type
    pub chunktype: [u8; 4],
}

fn find_null(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())
}

impl PngChunkRef {
    /// Read the length and type of a chunk from a [Read]'able stream to make a chunk reference
    ///
    /// This leaves the stream at the start of chunk data.
    pub(crate) fn from_stream<R>(stream: &mut R) -> Result<Self, std::io::Error>
    where
        R: Read + Seek,
    {
        let position = stream.stream_position()?;

        let mut buf4 = [0_u8; 4];
        stream.read_exact(&mut buf4)?;
        let length = u32::from_be_bytes(buf4);

        let mut chunktype = [0_u8; 4];
        stream.read_exact(&mut chunktype)?;

        Ok(Self {
            position,
            length,
            chunktype,
        })
    }

    /// Convert the chunk type bytes to a string that can be compared and printed much more easily
    #[inline]
    pub fn type_str(&self) -> &str {
        str::from_utf8(&self.chunktype).unwrap_or("")
    }

    /// Is this chunk necessary for successful display of the contents of
    /// the datastream (false) or not (true)? Derived from the case of the
    /// first character of the chunk type.
    #[inline]
    pub fn is_ancillary(&self) -> bool {
        self.chunktype[0] & 0x20 > 0
    }

    /// Is this chunk defined publically (false) or privately (true)? Derived
    /// from the case of the second character of the chunk type.
    #[inline]
    pub fn is_private(&self) -> bool {
        self.chunktype[1] & 0x20 > 0
    }

    /// Reserved for future use. All chunks should have this set to false.
    /// Derived from the case of the third character of the chunk type.
    #[inline]
    pub fn is_reserved(&self) -> bool {
        self.chunktype[2] & 0x20 > 0
    }

    /// Is this chunk safe to copy to a new datastream without processing?
    /// Derived from the case of the fourth character of the chunk type.
    #[inline]
    pub fn is_safe_to_copy(&self) -> bool {
        self.chunktype[3] & 0x20 > 0
    }

    /// Read just the sequence number of an fcTL or fdAT chunk
    pub(crate) fn read_fctl_fdat_sequence_number<R>(
        &self,
        stream: &mut R,
    ) -> Result<u32, std::io::Error>
    where
        R: Read + Seek,
    {
        stream.seek(SeekFrom::Start(self.position))?;
        let mut chunkstream = stream.take(self.length as u64);
        match &self.chunktype {
            b"fcTL" | b"fdAT" => {
                let mut buf4 = [0_u8; 4];
                chunkstream.read_exact(&mut buf4)?;
                Ok(u32::from_be_bytes(buf4))
            }

            _ => Err(std::io::Error::other(format!(
                "PNG: Chunk type ({:?}) is not an fcTL or fdAT",
                self.chunktype
            ))),
        }
    }

    /// Read the chunk data and parse it into a PngChunkData enum
    ///
    /// This also checks the chunk CRC value.
    pub(crate) fn read_chunk<R>(
        &self,
        stream: &mut R,
        ihdr: Option<&PngChunkData>,
    ) -> Result<PngChunkData, std::io::Error>
    where
        R: Read + Seek,
    {
        stream.seek(SeekFrom::Start(self.position + 4 + 4))?;
        let mut chunkstream = stream.take(self.length as u64);

        let mut data_crc = CRC::new();
        data_crc.consume(&self.chunktype);

        let chunk = match &self.chunktype {
            b"IHDR" => {
                let mut buf = [0_u8; 13];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Ihdr {
                    width: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    height: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                    bit_depth: buf[8],
                    colour_type: buf[9].try_into().map_err(to_io_error)?,
                    compression_method: buf[10].try_into().map_err(to_io_error)?,
                    filter_method: buf[11].try_into().map_err(to_io_error)?,
                    interlace_method: buf[12].try_into().map_err(to_io_error)?,
                })
            }

            b"PLTE" => Ok(PngChunkData::Plte(Box::new(
                (0..self.length / 3)
                    .map(|_| {
                        let mut buf = [0_u8; 3];
                        chunkstream.read_exact(&mut buf)?;
                        data_crc.consume(&buf);
                        Ok(PngPaletteEntry {
                            red: buf[0],
                            green: buf[1],
                            blue: buf[2],
                        })
                    })
                    .collect::<Result<Vec<_>, std::io::Error>>()?,
            ))),

            b"IDAT" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Idat(Box::new(data)))
            }

            b"IEND" => Ok(PngChunkData::Iend),

            b"tRNS" => {
                if ihdr.is_none() {
                    return Err(std::io::Error::other("PNG: Unset ihdr".to_string()));
                }

                if let PngChunkData::Ihdr { colour_type, .. } = ihdr.unwrap() {
                    match *colour_type {
                        PngColourType::Greyscale => {
                            let mut buf = [0_u8; 2];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PngChunkData::Trns {
                                data: PngTrnsType::Greyscale {
                                    value: u16::from_be_bytes(buf),
                                },
                            })
                        }

                        PngColourType::TrueColour => {
                            let mut buf = [0_u8; 6];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PngChunkData::Trns {
                                data: PngTrnsType::TrueColour {
                                    red: u16::from_be_bytes(
                                        buf[0..2].try_into().map_err(to_io_error)?,
                                    ),
                                    green: u16::from_be_bytes(
                                        buf[2..4].try_into().map_err(to_io_error)?,
                                    ),
                                    blue: u16::from_be_bytes(
                                        buf[4..6].try_into().map_err(to_io_error)?,
                                    ),
                                },
                            })
                        }

                        PngColourType::IndexedColour => {
                            let mut values = vec![0_u8; self.length as usize];
                            chunkstream.read_exact(&mut values)?;
                            data_crc.consume(&values);

                            Ok(PngChunkData::Trns {
                                data: PngTrnsType::IndexedColour { values },
                            })
                        }

                        _ => Err(std::io::Error::other(format!(
                            "PNG: Invalid colour type ({}) in ihdr",
                            *colour_type as u8
                        ))),
                    }
                } else {
                    Err(std::io::Error::other(
                        "PNG: Wrong chunk type passed as ihdr",
                    ))
                }
            }

            b"gAMA" => {
                let mut buf = [0_u8; 4];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Gama {
                    gamma: u32::from_be_bytes(buf),
                })
            }

            b"cHRM" => {
                let mut data = [0_u8; 32];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Chrm(Box::new(Chrm {
                    white_x: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
                    white_y: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
                    red_x: u32::from_be_bytes(data[8..12].try_into().map_err(to_io_error)?),
                    red_y: u32::from_be_bytes(data[12..16].try_into().map_err(to_io_error)?),
                    green_x: u32::from_be_bytes(data[16..20].try_into().map_err(to_io_error)?),
                    green_y: u32::from_be_bytes(data[20..24].try_into().map_err(to_io_error)?),
                    blue_x: u32::from_be_bytes(data[24..28].try_into().map_err(to_io_error)?),
                    blue_y: u32::from_be_bytes(data[28..32].try_into().map_err(to_io_error)?),
                })))
            }

            b"iCCP" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let name_end = find_null(&data);
                Ok(PngChunkData::Iccp(Box::new(Iccp {
                    name: data[0..name_end].iter().map(|b| *b as char).collect(),
                    compression_method: data[name_end].try_into().map_err(to_io_error)?,
                    compressed_profile: data[name_end + 2..].to_vec(),
                })))
            }

            b"sBIT" => {
                if ihdr.is_none() {
                    return Err(std::io::Error::other("PNG: Unset ihdr".to_string()));
                }

                if let PngChunkData::Ihdr { colour_type, .. } = ihdr.unwrap() {
                    match colour_type {
                        PngColourType::Greyscale => {
                            let mut buf = [0_u8; 1];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PngChunkData::Sbit {
                                bits: PngSbitType::Greyscale { grey_bits: buf[0] },
                            })
                        }

                        PngColourType::TrueColour | PngColourType::IndexedColour => {
                            let mut buf = [0_u8; 3];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PngChunkData::Sbit {
                                bits: PngSbitType::Colour {
                                    red_bits: buf[0],
                                    green_bits: buf[1],
                                    blue_bits: buf[2],
                                },
                            })
                        }

                        PngColourType::GreyscaleAlpha => {
                            let mut buf = [0_u8; 2];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PngChunkData::Sbit {
                                bits: PngSbitType::GreyscaleAlpha {
                                    grey_bits: buf[0],
                                    alpha_bits: buf[1],
                                },
                            })
                        }

                        PngColourType::TrueColourAlpha => {
                            let mut buf = [0_u8; 4];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PngChunkData::Sbit {
                                bits: PngSbitType::TrueColourAlpha {
                                    red_bits: buf[0],
                                    green_bits: buf[1],
                                    blue_bits: buf[2],
                                    alpha_bits: buf[3],
                                },
                            })
                        }
                    }
                } else {
                    Err(std::io::Error::other(
                        "PNG: Wrong chunk type passed as ihdr",
                    ))
                }
            }

            b"sRGB" => {
                let mut buf = [0_u8; 1];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Srgb {
                    rendering_intent: buf[0].try_into().map_err(to_io_error)?,
                })
            }

            b"cICP" => {
                let mut buf = [0_u8; 4];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Cicp {
                    colour_primaries: buf[0].into(),
                    transfer_function: buf[1].into(),
                    matrix_coeffs: buf[2].into(),
                    video_full_range: buf[3] > 0,
                })
            }

            b"mDCV" => {
                let mut buf = [0_u8; 24];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Mdcv(Box::new(Mdcv {
                    red_x: u16::from_be_bytes(buf[0..2].try_into().map_err(to_io_error)?),
                    red_y: u16::from_be_bytes(buf[2..4].try_into().map_err(to_io_error)?),
                    green_x: u16::from_be_bytes(buf[4..6].try_into().map_err(to_io_error)?),
                    green_y: u16::from_be_bytes(buf[6..8].try_into().map_err(to_io_error)?),
                    blue_x: u16::from_be_bytes(buf[8..10].try_into().map_err(to_io_error)?),
                    blue_y: u16::from_be_bytes(buf[10..12].try_into().map_err(to_io_error)?),
                    white_x: u16::from_be_bytes(buf[12..14].try_into().map_err(to_io_error)?),
                    white_y: u16::from_be_bytes(buf[14..16].try_into().map_err(to_io_error)?),
                    max_lum: u32::from_be_bytes(buf[16..20].try_into().map_err(to_io_error)?),
                    min_lum: u32::from_be_bytes(buf[20..24].try_into().map_err(to_io_error)?),
                })))
            }

            b"cLLI" => {
                let mut buf = [0_u8; 8];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Clli(Box::new(Clli {
                    max_cll: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    max_fall: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                })))
            }

            b"tEXt" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                Ok(PngChunkData::Text(Box::new(Text {
                    keyword: data[0..keyword_end].iter().map(|b| *b as char).collect(),
                    string: data[keyword_end + 1..].iter().map(|b| *b as char).collect(),
                })))
            }

            b"zTXt" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                Ok(PngChunkData::Ztxt(Box::new(Ztxt {
                    keyword: data[0..keyword_end].iter().map(|b| *b as char).collect(),
                    compression_method: data[keyword_end + 1].try_into().map_err(to_io_error)?,
                    compressed_string: data[keyword_end + 2..].to_vec(),
                })))
            }

            b"iTXt" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                let language_end = find_null(&data[keyword_end + 3..]) + keyword_end + 3;
                let tkeyword_end = find_null(&data[language_end + 1..]) + language_end + 1;

                Ok(PngChunkData::Itxt(Box::new(Itxt {
                    keyword: data[0..keyword_end].iter().map(|b| *b as char).collect(),
                    compressed: data[keyword_end + 1] > 0,
                    compression_method: data[keyword_end + 2].try_into().map_err(to_io_error)?,
                    language: data[keyword_end + 3..language_end]
                        .iter()
                        .map(|b| *b as char)
                        .collect(),
                    translated_keyword: String::from_utf8(
                        data[language_end + 1..tkeyword_end].to_vec(),
                    )
                    .map_err(to_io_error)?,
                    compressed_string: data[tkeyword_end + 1..].to_vec(),
                })))
            }

            b"bKGD" => {
                if ihdr.is_none() {
                    return Err(std::io::Error::other("PNG: Unset ihdr".to_string()));
                }

                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                if let PngChunkData::Ihdr { colour_type, .. } = ihdr.unwrap() {
                    match colour_type {
                        PngColourType::Greyscale | PngColourType::GreyscaleAlpha => {
                            if self.length != 2 {
                                return Err(std::io::Error::other(format!(
                                    "PNG: Invalid length of bKGD chunk ({})",
                                    self.length
                                )));
                            }

                            Ok(PngChunkData::Bkgd {
                                data: PngBkgdType::Greyscale {
                                    value: u16::from_be_bytes(
                                        data[0..2].try_into().map_err(to_io_error)?,
                                    ),
                                },
                            })
                        }

                        PngColourType::TrueColour | PngColourType::TrueColourAlpha => {
                            if self.length != 6 {
                                return Err(std::io::Error::other(format!(
                                    "Png: Invalid length of bKGD chunk ({})",
                                    self.length
                                )));
                            }

                            Ok(PngChunkData::Bkgd {
                                data: PngBkgdType::TrueColour {
                                    red: u16::from_be_bytes(
                                        data[0..2].try_into().map_err(to_io_error)?,
                                    ),
                                    green: u16::from_be_bytes(
                                        data[2..4].try_into().map_err(to_io_error)?,
                                    ),
                                    blue: u16::from_be_bytes(
                                        data[4..6].try_into().map_err(to_io_error)?,
                                    ),
                                },
                            })
                        }

                        PngColourType::IndexedColour => {
                            if self.length != 1 {
                                return Err(std::io::Error::other(format!(
                                    "Png: Invalid length of bKGD chunk ({})",
                                    self.length
                                )));
                            }

                            Ok(PngChunkData::Bkgd {
                                data: PngBkgdType::IndexedColour { index: data[0] },
                            })
                        }
                    }
                } else {
                    Err(std::io::Error::other(
                        "PNG: Wrong chunk type passed as ihdr",
                    ))
                }
            }

            b"hIST" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Hist(Box::new(
                    (0..self.length / 2)
                        .map(|n| {
                            let start = n as usize * 2;
                            Ok(u16::from_be_bytes(
                                data[start..start + 2].try_into().map_err(to_io_error)?,
                            ))
                        })
                        .collect::<Result<Vec<_>, std::io::Error>>()?,
                )))
            }

            b"pHYs" => {
                let mut buf = [0_u8; 9];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Phys {
                    x_pixels_per_unit: u32::from_be_bytes(
                        buf[0..4].try_into().map_err(to_io_error)?,
                    ),
                    y_pixels_per_unit: u32::from_be_bytes(
                        buf[4..8].try_into().map_err(to_io_error)?,
                    ),
                    unit: buf[8].try_into().map_err(to_io_error)?,
                })
            }

            b"eXIf" => {
                let mut profile = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut profile)?;
                data_crc.consume(&profile);

                Ok(PngChunkData::Exif(Box::new(profile)))
            }

            b"sPLT" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let name_end = find_null(&data);
                let depth = data[name_end + 1];
                let entry_size = ((depth / 8) * 4) + 2;
                let num_entries = (self.length as usize - name_end - 1) / (entry_size as usize);

                Ok(PngChunkData::Splt(Box::new(Splt {
                    name: data[0..name_end].iter().map(|b| *b as char).collect(),
                    depth,
                    palette: (0..num_entries)
                        .map(|i| {
                            let start = name_end + 2 + (i * entry_size as usize);
                            if depth == 8 {
                                Ok(PngSuggestedPaletteEntry {
                                    red: data[start] as u16,
                                    green: data[start + 1] as u16,
                                    blue: data[start + 2] as u16,
                                    alpha: data[start + 3] as u16,
                                    frequency: u16::from_be_bytes(
                                        data[start + 4..start + 6]
                                            .try_into()
                                            .map_err(to_io_error)?,
                                    ),
                                })
                            } else {
                                Ok(PngSuggestedPaletteEntry {
                                    red: u16::from_be_bytes(
                                        data[start..start + 2].try_into().map_err(to_io_error)?,
                                    ),
                                    green: u16::from_be_bytes(
                                        data[start + 2..start + 4]
                                            .try_into()
                                            .map_err(to_io_error)?,
                                    ),
                                    blue: u16::from_be_bytes(
                                        data[start + 4..start + 6]
                                            .try_into()
                                            .map_err(to_io_error)?,
                                    ),
                                    alpha: u16::from_be_bytes(
                                        data[start + 6..start + 8]
                                            .try_into()
                                            .map_err(to_io_error)?,
                                    ),
                                    frequency: u16::from_be_bytes(
                                        data[start + 8..start + 10]
                                            .try_into()
                                            .map_err(to_io_error)?,
                                    ),
                                })
                            }
                        })
                        .collect::<Result<Vec<_>, std::io::Error>>()?,
                })))
            }

            b"tIME" => {
                let mut buf = [0_u8; 7];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Time {
                    year: u16::from_be_bytes(buf[0..2].try_into().map_err(to_io_error)?),
                    month: buf[2],
                    day: buf[3],
                    hour: buf[4],
                    minute: buf[5],
                    second: buf[6],
                })
            }

            // Animation information
            b"acTL" => {
                let mut buf = [0_u8; 8];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Actl {
                    num_frames: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    num_plays: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                })
            }

            b"fcTL" => {
                let mut buf = [0_u8; 26];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Fctl(Box::new(Fctl {
                    sequence_number: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    width: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                    height: u32::from_be_bytes(buf[8..12].try_into().map_err(to_io_error)?),
                    x_offset: u32::from_be_bytes(buf[12..16].try_into().map_err(to_io_error)?),
                    y_offset: u32::from_be_bytes(buf[16..20].try_into().map_err(to_io_error)?),
                    delay_num: u16::from_be_bytes(buf[20..22].try_into().map_err(to_io_error)?),
                    delay_den: u16::from_be_bytes(buf[22..24].try_into().map_err(to_io_error)?),
                    dispose_op: buf[24].try_into().map_err(to_io_error)?,
                    blend_op: buf[24].try_into().map_err(to_io_error)?,
                })))
            }

            b"fdAT" => {
                let mut buf = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Fdat(Box::new(Fdat {
                    sequence_number: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    frame_data: buf[4..].to_vec(),
                })))
            }

            // Extensions
            b"oFFs" => {
                let mut buf = [0_u8; 9];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Offs {
                    x: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    y: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                    unit: buf[8].try_into().map_err(to_io_error)?,
                })
            }

            b"pCAL" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let name_end = find_null(&data);
                let num_parameters = data[name_end + 9];
                let unit_end = find_null(&data[name_end + 10..]) + name_end + 10;
                let mut parameters = Vec::with_capacity(num_parameters as usize);

                // TODO: can this be done with an iterator?
                let mut prev_end = unit_end;
                for _ in 0..num_parameters {
                    let param_end = find_null(&data[prev_end..]) + prev_end;
                    parameters.push(
                        data[prev_end..param_end]
                            .iter()
                            .map(|b| *b as char)
                            .collect(),
                    );
                    prev_end = param_end;
                }

                Ok(PngChunkData::Pcal(Box::new(Pcal {
                    name: data[0..name_end].iter().map(|b| *b as char).collect(),
                    original_zero: u32::from_be_bytes(
                        data[name_end..name_end + 4]
                            .try_into()
                            .map_err(to_io_error)?,
                    ),
                    original_max: u32::from_be_bytes(
                        data[name_end + 4..name_end + 8]
                            .try_into()
                            .map_err(to_io_error)?,
                    ),
                    equation_type: data[name_end + 8],
                    unit_name: data[name_end + 10..unit_end]
                        .iter()
                        .map(|b| *b as char)
                        .collect(),
                    parameters,
                })))
            }

            b"sCAL" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let width_end = find_null(&data[1..]) + 1;
                let height_end = find_null(&data[width_end..]) + width_end;

                Ok(PngChunkData::Scal(Box::new(Scal {
                    unit: data[0].try_into().map_err(to_io_error)?,
                    pixel_width: data[1..width_end].iter().map(|b| *b as char).collect(),
                    pixel_height: data[width_end..height_end]
                        .iter()
                        .map(|b| *b as char)
                        .collect(),
                })))
            }

            b"gIFg" => {
                let mut buf = [0_u8; 4];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Gifg {
                    disposal_method: buf[0].into(),
                    user_input: buf[1] > 0,
                    delay_time: u16::from_be_bytes(buf[2..].try_into().map_err(to_io_error)?),
                })
            }

            b"gIFx" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Gifx(Box::new(Gifx {
                    app_id: data[0..8].iter().map(|b| *b as char).collect(),
                    app_auth: [data[8], data[9], data[10]],
                    app_data: data[11..].to_vec(),
                })))
            }

            b"sTER" => {
                let mut buf = [0_u8; 1];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Ster { mode: buf[0] })
            }

            // JNG chunks
            b"Jhdr" => {
                let mut buf = [0_u8; 16];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PngChunkData::Jhdr {
                    width: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    height: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                    colour_type: buf[8].try_into().map_err(to_io_error)?,
                    image_sample_depth: buf[9].try_into().map_err(to_io_error)?,
                    image_compression_method: buf[10].try_into().map_err(to_io_error)?,
                    image_interlace_method: buf[11].try_into().map_err(to_io_error)?,
                    alpha_sample_depth: buf[12].try_into().map_err(to_io_error)?,
                    alpha_compression_method: buf[13].try_into().map_err(to_io_error)?,
                    alpha_filter_method: buf[14].try_into().map_err(to_io_error)?,
                    alpha_interlace_method: buf[15].try_into().map_err(to_io_error)?,
                })
            }

            b"JDAT" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Jdat(Box::new(data)))
            }

            b"JDAA" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Jdaa(Box::new(data)))
            }

            b"JSEP" => Ok(PngChunkData::Jsep),

            _ => Err(std::io::Error::other(format!(
                "PNG: Unhandled chunk type ({:?})",
                self.chunktype
            ))),
        }?;

        let mut buf4 = [0_u8; 4];
        stream.read_exact(&mut buf4)?;
        let crc = u32::from_be_bytes(buf4);
        if crc != data_crc.value() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "PNG: Read CRC ({:#x}) doesn't match the computed one ({:#x})",
                    crc,
                    data_crc.value()
                ),
            ));
        }

        Ok(chunk)
    }
}

/// A frame in an APNG file
#[derive(Clone, Default, Debug)]
pub struct ApngFrame {
    /// The fcTL chunk defining the frame
    pub fctl: PngChunkRef,

    /// The fdAT chunk(s) containing the frame data
    pub fdats: Vec<PngChunkRef>,
}
