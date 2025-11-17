/*
  png-container
  Copyright (C) 2025 Ian Tester

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

//! Colour space chunks

use std::io::Read;

use flate2::{
    Compression,
    bufread::{ZlibDecoder, ZlibEncoder},
};
use uom::si::{f64::Luminance, luminance::candela_per_square_meter};

use crate::chunks::find_null;
use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

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
    pub(crate) const TYPE: [u8; 4] = *b"cHRM";

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

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 32];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            white_x: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            white_y: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            red_x: u32::from_be_bytes(data[8..12].try_into().map_err(to_io_error)?),
            red_y: u32::from_be_bytes(data[12..16].try_into().map_err(to_io_error)?),
            green_x: u32::from_be_bytes(data[16..20].try_into().map_err(to_io_error)?),
            green_y: u32::from_be_bytes(data[20..24].try_into().map_err(to_io_error)?),
            blue_x: u32::from_be_bytes(data[24..28].try_into().map_err(to_io_error)?),
            blue_y: u32::from_be_bytes(data[28..32].try_into().map_err(to_io_error)?),
        })
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

/// Image gamma
#[derive(Clone, Copy, Debug, Default)]
pub struct Gama {
    /// Gamma value, scaled by 100000
    pub gamma: u32,
}

impl Gama {
    pub(crate) const TYPE: [u8; 4] = *b"gAMA";

    /// Constructor
    pub fn new(gamma: f64) -> Self {
        Self {
            gamma: (gamma * 100000.0).round() as u32,
        }
    }

    /// Set the gamma value
    pub fn set_gamma(&mut self, gamma: f64) {
        self.gamma = (gamma * 100000.0).round() as u32
    }

    /// Gamma value
    pub fn gamma(self) -> f64 {
        self.gamma as f64 / 100000.0
    }

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 4];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            gamma: u32::from_be_bytes(data),
        })
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
    pub(crate) const TYPE: [u8; 4] = *b"iCCP";

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

    /// Read contents from a stream
    pub fn from_stream<R>(
        stream: &mut R,
        length: u32,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = vec![0_u8; length as usize];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        let name_end = find_null(&data);
        Ok(Self {
            name: data[0..name_end].iter().map(|b| *b as char).collect(),
            compression_method: data[name_end].try_into().map_err(to_io_error)?,
            compressed_profile: data[name_end + 2..].to_vec(),
        })
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

/// Significant bits
#[derive(Clone, Copy, Debug)]
pub enum Sbit {
    Greyscale {
        grey_bits: u8,
    },

    Colour {
        red_bits: u8,
        green_bits: u8,
        blue_bits: u8,
    },

    GreyscaleAlpha {
        grey_bits: u8,
        alpha_bits: u8,
    },

    TrueColourAlpha {
        red_bits: u8,
        green_bits: u8,
        blue_bits: u8,
        alpha_bits: u8,
    },
}

impl Sbit {
    pub(crate) const TYPE: [u8; 4] = *b"sBIT";

    /// Read contents from a stream
    pub fn from_stream<R>(
        stream: &mut R,
        length: u32,
        colour_type: PngColourType,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = vec![0_u8; length as usize];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        match colour_type {
            PngColourType::Greyscale => Ok(Self::Greyscale { grey_bits: data[0] }),

            PngColourType::TrueColour | PngColourType::IndexedColour => Ok(Self::Colour {
                red_bits: data[0],
                green_bits: data[1],
                blue_bits: data[2],
            }),

            PngColourType::GreyscaleAlpha => Ok(Self::GreyscaleAlpha {
                grey_bits: data[0],
                alpha_bits: data[1],
            }),

            PngColourType::TrueColourAlpha => Ok(Self::TrueColourAlpha {
                red_bits: data[0],
                green_bits: data[1],
                blue_bits: data[2],
                alpha_bits: data[3],
            }),
        }
    }
}

/// Standard RGB colour space
#[derive(Clone, Copy, Debug)]
pub struct Srgb {
    pub rendering_intent: PngRenderingIntent,
}

impl Srgb {
    pub(crate) const TYPE: [u8; 4] = *b"sRGB";

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 1];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            rendering_intent: data[0].try_into().map_err(to_io_error)?,
        })
    }
}

/// Coding-independent code points for video signal type identification
#[derive(Clone, Copy, Debug)]
pub struct Cicp {
    pub colour_primaries: ColourPrimaries,
    pub transfer_function: TransferFunction,
    pub matrix_coeffs: MatrixCoefficients,
    pub video_full_range: bool,
}

impl Cicp {
    pub(crate) const TYPE: [u8; 4] = *b"cICP";

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 4];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            colour_primaries: data[0].into(),
            transfer_function: data[1].into(),
            matrix_coeffs: data[2].into(),
            video_full_range: data[3] > 0,
        })
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
    pub(crate) const TYPE: [u8; 4] = *b"mDCV";

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

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 24];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            red_x: u16::from_be_bytes(data[0..2].try_into().map_err(to_io_error)?),
            red_y: u16::from_be_bytes(data[2..4].try_into().map_err(to_io_error)?),
            green_x: u16::from_be_bytes(data[4..6].try_into().map_err(to_io_error)?),
            green_y: u16::from_be_bytes(data[6..8].try_into().map_err(to_io_error)?),
            blue_x: u16::from_be_bytes(data[8..10].try_into().map_err(to_io_error)?),
            blue_y: u16::from_be_bytes(data[10..12].try_into().map_err(to_io_error)?),
            white_x: u16::from_be_bytes(data[12..14].try_into().map_err(to_io_error)?),
            white_y: u16::from_be_bytes(data[14..16].try_into().map_err(to_io_error)?),
            max_lum: u32::from_be_bytes(data[16..20].try_into().map_err(to_io_error)?),
            min_lum: u32::from_be_bytes(data[20..24].try_into().map_err(to_io_error)?),
        })
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
    pub(crate) const TYPE: [u8; 4] = *b"cLLI";

    /// Constructor
    pub fn new(max_cll: Luminance, max_fall: Luminance) -> Self {
        Self {
            max_cll: (max_cll.get::<candela_per_square_meter>() * 10000.0) as u32,
            max_fall: (max_fall.get::<candela_per_square_meter>() * 10000.0) as u32,
        }
    }

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 8];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            max_cll: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            max_fall: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
        })
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
