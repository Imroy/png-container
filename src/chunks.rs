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
use std::str;

use crate::types::*;

/// Palette entry
#[derive(Clone, Debug)]
pub struct PNGPaletteEntry {
    red: u8,
    green: u8,
    blue: u8,

}


/// Colour type of image
#[derive(Copy, Clone, Debug)]
pub enum PNGColourType {
    /// Greyscale image - allowed depths of 1, 2, 4, 8, or 16 bits per component
    Greyscale = 0,

    /// RGB colour image - allowed depths of 8 or 16 bits per component
    TrueColour = 2,

    /// Indexed colour image - allowed depths of 1, 2, 4, or 8 bits per index
    IndexedColour,

    /// Greyscale image with alpha - allowed depths of 8 or 16 bits per component
    GreyscaleAlpha,

    /// RGB colour image with alpha - allowed depths of 8 or 16 bits per component
    TrueColourAlpha = 6,

}

impl TryFrom<u8> for PNGColourType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGColourType::Greyscale as u8 => Ok(PNGColourType::Greyscale),
            x if x == PNGColourType::TrueColour as u8 => Ok(PNGColourType::TrueColour),
            x if x == PNGColourType::IndexedColour as u8 => Ok(PNGColourType::IndexedColour),
            x if x == PNGColourType::GreyscaleAlpha as u8 => Ok(PNGColourType::GreyscaleAlpha),
            x if x == PNGColourType::TrueColourAlpha as u8 => Ok(PNGColourType::TrueColourAlpha),
            _ => Err(std::io::Error::other(format!("PNG: Invalid value of colour type ({})", val))),
        }
    }
}


/// Compression type(s)
#[derive(Copy, Clone, Debug)]
pub enum PNGCompressionType {
    /// DEFLATE
    Zlib = 0,

}

impl TryFrom<u8> for PNGCompressionType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGCompressionType::Zlib as u8 => Ok(PNGCompressionType::Zlib),
            _ => Err(std::io::Error::other(format!("PNG: Invalid value of compression method ({})", val))),
        }
    }
}


/// Filter types
#[derive(Copy, Clone, Debug)]
pub enum PNGFilterType {
    /// Adaptive filtering with five basic filter types
    Adaptive = 0,

}

impl TryFrom<u8> for PNGFilterType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGFilterType::Adaptive as u8 => Ok(PNGFilterType::Adaptive),
            _ => Err(std::io::Error::other(format!("PNG: Invalid value of filter method ({})", val))),
        }
    }
}


/// Interlacing types
#[derive(Copy, Clone, Debug)]
pub enum PNGInterlaceType {
    /// No interlacing
    None = 0,

    /// Adam7 interlacing
    Adam7,

}

impl TryFrom<u8> for PNGInterlaceType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGInterlaceType::None as u8 => Ok(PNGInterlaceType::None),
            x if x == PNGInterlaceType::Adam7 as u8 => Ok(PNGInterlaceType::Adam7),
            _ => Err(std::io::Error::other(format!("PNG: Invalid value of interlace method ({})", val))),
        }
    }
}


/// Contents of tRNS chunk
#[derive(Clone, Debug)]
pub enum PNGtRNSType {
    Greyscale {
        value: u16,
    },

    TrueColour {
        red: u16,
        green: u16,
        blue: u16,
    },

    IndexedColour {
        values: Vec<u8>,
    },

}


/// Contents of sBIT chunk
#[derive(Copy, Clone, Debug)]
pub enum PNGsBITType {
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


/// ICC rendering intent
#[derive(Copy, Clone, Debug)]
pub enum PNGRenderingIntent {
    Perceptual,
    Relative_colorimetric,
    Saturation,
    Absolute_colorimetric,
}

impl TryFrom<u8> for PNGRenderingIntent {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGRenderingIntent::Perceptual as u8 => Ok(PNGRenderingIntent::Perceptual),
            x if x == PNGRenderingIntent::Relative_colorimetric as u8 => Ok(PNGRenderingIntent::Relative_colorimetric),
            x if x == PNGRenderingIntent::Saturation as u8 => Ok(PNGRenderingIntent::Saturation),
            x if x == PNGRenderingIntent::Absolute_colorimetric as u8 => Ok(PNGRenderingIntent::Absolute_colorimetric),
            _ => Err(std::io::Error::other(format!("PNG: Invalid value of rendering intent ({})", val))),
        }
    }

}


/// Contents of bKGD chunk
#[derive(Copy, Clone, Debug)]
pub enum PNGbKGDType {
    Greyscale {
        value: u16,
    },

    TrueColour {
        red: u16,
        green: u16,
        blue: u16,
    },

    IndexedColour {
        index: u8,
    },

}


/// Unit type used in several chunks
#[derive(Copy, Clone, Debug)]
pub enum PNGUnitType {
    Unknown = 0,

    Metre = 1,

}

impl TryFrom<u8> for PNGUnitType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGUnitType::Unknown as u8 => Ok(PNGUnitType::Unknown),
            x if x == PNGUnitType::Metre as u8 => Ok(PNGUnitType::Metre),
            _ => Err(std::io::Error::other(format!("PNG: Invalid value of unit ({})", val))),
        }
    }

}


/// Enum of PNG chunk types and the data they hold
#[derive(Clone, Debug)]
pub enum PNGChunkData {
    // Critical chunks

    /// Image header
    IHDR {
        /// Width of image in pixels
        width: u32,

        /// Height of image in pixels
        height: u32,

        /// Number of bits per sample
        bit_depth: u8,

        /// Colour type
        colour_type: PNGColourType,

        /// Compression method
        compression_method: PNGCompressionType,

        /// Filter method
        filter_method: PNGFilterType,

        /// Interlace method
        interlace_method: PNGInterlaceType,
    },

    /// Palette
    PLTE {
        entries: Vec<PNGPaletteEntry>,
    },

    /// Image data
    IDAT {
        data: Vec<u8>,
    },

    /// Image end
    IEND,

    // Transparency information

    /// Transparency
    TRNS {
        data: PNGtRNSType,
    },

    // Colour space information

    /// Primary chromaticities and white point
    ///
    /// Values are scaled by 100000
    CHRM {
        white_x: u32,
        white_y: u32,
        red_x: u32,
        red_y: u32,
        green_x: u32,
        green_y: u32,
        blue_x: u32,
        blue_y: u32,
    },

    /// Image gamma
    ///
    /// Value is scaled by 100000
    GAMA {
        gamma: u32,
    },

    /// Embedded ICC profile
    ICCP {
        name: String,
        compression_method: PNGCompressionType,
        compressed_profile: Vec<u8>,
    },

    /// Significant bits
    SBIT {
        bits: PNGsBITType,
    },

    /// Standard RGB colour space
    SRGB {
        rendering_intent: PNGRenderingIntent,
    },

    /// Coding-independent code points for video signal type identification
    CICP {
        colour_primaries: u8,
        transfer_function: u8,
        matrix_coeffs: u8,
        video_full_range: bool,
    },

    // Textual information

    /// Textual data
    TEXT {
        keyword: String,
        string: String,
    },

    /// Compressed textual data
    ZTXT {
        keyword: String,
        compression_method: PNGCompressionType,
        compressed_string: Vec<u8>,
    },

    /// International textual data
    ITXT {
        keyword: String,
        compressed: bool,
        compression_method: PNGCompressionType,
        language: String,
        translated_keyword: String,
        compressed_string: Vec<u8>,
    },

    // Miscellaneous information

    /// Background colour
    BKGD {
        data: PNGbKGDType,
    },

    /// Image histogram
    HIST {
        frequencies: Vec<u16>,
    },

    /// Physical pixel dimensions
    PHYS {
        x_pixels_per_unit: u32,
        y_pixels_per_unit: u32,
        unit: PNGUnitType,
    },

    /// Suggested palette
    SPLT {
        name: String,
        depth: u8,
        // TODO
    },

    /// Exchangeable Image File (Exif) Profile
    EXIF {
        profile: Vec<u8>,
    },

    // Time stamp information

    /// Image last-modification time
    TIME {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    },

    // Animation information

    // TODO

    // Extensions

    /// Image offset
    OFFS {
        x: u32,
        y: u32,
        unit: PNGUnitType,
    },

    /// Calibration of pixel values
    PCAL {
        name: String,
        original_zero: u32,
        original_max: u32,
        equation_type: u8,
        num_parameters: u8,
        unit_name: String,
        parameters: Vec<String>,
    },

    /// Physical scale of image subject
    SCAL {
        unit: PNGUnitType,
        pixel_width: String,
        pixel_height: String,
    },

    /// GIF Graphic Control Extension
    GIFG {
        disposal_method: u8,
        user_input: bool,
        delay_time: u16,
    },

    /// GIF Application Extension
    GIFX {
        app_id: [ u8; 8 ],
        app_code: [ u8; 3 ],
        app_data: Vec<u8>,
    },

    /// Indicator of Stereo Image
    STER {
        mode: u8,
    },

}

impl PNGChunkData {

}


/// A chunk in a PNG file
#[derive(Copy, Clone, Debug)]
pub struct PNGChunk {
    /// The position in the stream/file for this chunk
    pub position: u64,

    pub length: u32,

    pub chunktype: [ u8; 4 ],

    pub crc: u32,

    /// Is this chunk necessary for successful display of the contents of
    /// the datastream (false) or not (true)? Derived from the case of the
    /// first character of the chunk type.
    pub ancillary: bool,

    /// Is this chunk defined publically (false) or privately (true)? Derived
    /// from the case of the second character of the chunk type.
    pub private: bool,

    /// Reserved for future use. All chunks should have this set to false.
    /// Derived from the case of the third character of the chunk type.
    pub reserved: bool,

    /// Is this chunk safe to copy to a new datastream without processing?
    /// Derived from the case of the fourth character of the chunk type.
    pub safe_to_copy: bool,

}

// because u32::from_be_bytes() only takes fixed-length arrays and it's too
// much of a PITA to convert from a slice.
fn u32_be(bytes: &[u8]) -> u32 {
    (bytes[3] as u32) | ((bytes[2] as u32) << 8) | ((bytes[1] as u32) << 16) | ((bytes[0] as u32) << 24)
}

impl PNGChunk {
    /// Convert the chunk type bytes to a string that can be compared and printed much more easily
    #[inline]
    pub fn type_str(&self) -> &str {
        str::from_utf8(&self.chunktype).unwrap_or("")
    }

    pub fn read_chunk<R>(&self, stream: &mut R) -> Result<PNGChunkData, std::io::Error>
        where R: Read + Seek
    {
        stream.seek(SeekFrom::Start(self.position + 8))?;
        let mut chunkstream = stream.take(self.length as u64);

        match self.type_str() {
            "IHDR" => {
                let mut buf = Vec::with_capacity(13);
                chunkstream.read_to_end(&mut buf)?;
                Ok(PNGChunkData::IHDR {
                    width: u32_be(&buf[0..4]),
                    height: u32_be(&buf[4..8]),
                    bit_depth: buf[8],
                    colour_type: buf[9].try_into()?,
                    compression_method: buf[10].try_into()?,
                    filter_method: buf[11].try_into()?,
                    interlace_method: buf[12].try_into()?,
                })
            },

            "PLTE" => {
                let num_entries = self.length / 3;
                let mut entries = Vec::with_capacity(num_entries as usize);
                for n in 0..num_entries {
                    let mut buf = [ 0_u8; 3 ];
                    chunkstream.read_exact(&mut buf)?;
                    entries.push(PNGPaletteEntry {
                        red: buf[0],
                        green: buf[1],
                        blue: buf[2],
                    });
                }

                Ok(PNGChunkData::PLTE {
                    entries,
                })
            },

            "IDAT" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;

                Ok(PNGChunkData::IDAT {
                    data,
                })
            }

            "IEND" => Ok(PNGChunkData::IEND),

            _ => Err(std::io::Error::other(format!("PNG: Unhandled chunk type ({})", self.type_str())))
        }
    }

}
