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

use std::io::Read;
use std::slice::Iter;
use std::str;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use inflate::inflate_bytes_zlib;
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
pub enum PNGChunkData {
    /// Empty type
    None,

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
        compression_method: PNGCompressionMethod,

        /// Filter method
        filter_method: PNGFilterMethod,

        /// Interlace method
        interlace_method: PNGInterlaceMethod,
    },

    /// Palette
    PLTE(Box<Vec<PNGPaletteEntry>>),

    /// Image data
    IDAT(Box<Vec<u8>>),

    /// Image end
    IEND,

    // Transparency information

    /// Transparency
    TRNS { data: PNGtRNSType },

    // Colour space information

    /// Primary chromaticities and white point
    CHRM(Box<CHRM>),

    /// Image gamma
    ///
    /// Value is scaled by 100000
    GAMA { gamma: u32 },

    /// Embedded ICC profile
    ICCP(Box<ICCP>),

    /// Significant bits
    SBIT { bits: PNGsBITType },

    /// Standard RGB colour space
    SRGB {
        rendering_intent: PNGRenderingIntent,
    },

    /// Coding-independent code points for video signal type identification
    CICP {
        colour_primaries: ColourPrimaries,
        transfer_function: TransferFunction,
        matrix_coeffs: MatrixCoefficients,
        video_full_range: bool,
    },

    /// Mastering Display Color Volume
    MDCV(Box<MDCV>),

    /// Content Light Level Information
    CLLI(Box<CLLI>),

    // Textual information
    /// Textual data
    TEXT(Box<TEXT>),

    /// Compressed textual data
    ZTXT(Box<ZTXT>),

    /// International textual data
    ITXT(Box<ITXT>),

    // Miscellaneous information

    /// Background colour
    BKGD { data: PNGbKGDType },

    /// Image histogram
    HIST(Box<Vec<u16>>),

    /// Physical pixel dimensions
    PHYS {
        x_pixels_per_unit: u32,
        y_pixels_per_unit: u32,
        unit: PNGUnitType,
    },

    /// Suggested palette
    SPLT(Box<SPLT>),

    /// Exchangeable Image File (Exif) Profile
    EXIF(Box<Vec<u8>>),

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

    /// Animation control
    ACTL { num_frames: u32, num_plays: u32 },

    /// Frame control
    FCTL(Box<FCTL>),

    /// Frame data
    FDAT(Box<FDAT>),

    // Extensions
    /// Image offset
    OFFS { x: u32, y: u32, unit: PNGUnitType },

    /// Calibration of pixel values
    PCAL(Box<PCAL>),

    /// Physical scale of image subject
    SCAL(Box<SCAL>),

    /// GIF Graphic Control Extension
    GIFG {
        // TODO: make this an enum
        disposal_method: u8,
        user_input: bool,
        delay_time: u16,
    },

    /// GIF Application Extension
    GIFX(Box<GIFX>),

    /// Indicator of Stereo Image
    STER { mode: u8 },

    // JNG chunks
    /// JNG header
    JHDR {
        /// Width of image in pixels
        width: u32,

        /// Height of image in pixels
        height: u32,

        /// Colour type
        colour_type: JNGColourType,

        /// Image sample depth
        image_sample_depth: JNGImageSampleDepth,

        /// Image compression method
        image_compression_method: JNGCompressionType,

        /// Image interlace method
        image_interlace_method: JNGInterlaceMethod,

        /// Alpha sample depth
        alpha_sample_depth: JNGAlphaSampleDepth,

        /// Alpha compression method
        alpha_compression_method: JNGCompressionType,

        /// Alpha channel filter method
        alpha_filter_method: PNGFilterMethod,

        /// Alpha interlace method
        alpha_interlace_method: JNGInterlaceMethod,
    },

    /// JNG image data
    JDAT(Box<Vec<u8>>),

    /// JNG alpha data
    JDAA(Box<Vec<u8>>),

    /// JNG image separator
    JSEP,
}

impl PNGChunkData {
    /// Return an iterator into the data of IDAT/fdAT/JDAT/JDAA chunks
    pub fn dat_data_iter(&self) -> Option<Iter<'_, u8>> {
        match self {
            PNGChunkData::IDAT(data) => Some(data.iter()),

            PNGChunkData::FDAT(fdat) => Some(fdat.frame_data.iter()),

            PNGChunkData::JDAT(data) => Some(data.iter()),

            PNGChunkData::JDAA(data) => Some(data.iter()),

            _ => None,
        }
    }

    /// Scaled white coordinates of the cHRM chunk
    pub fn chrm_white_coords(&self) -> Option<(f64, f64)> {
        if let PNGChunkData::CHRM(chrm) = self {
            return Some(chrm.white_coords());
        }

        None
    }

    /// Scaled red coordinates of the cHRM chunk
    pub fn chrm_red_coords(&self) -> Option<(f64, f64)> {
        if let PNGChunkData::CHRM(chrm) = self {
            return Some(chrm.red_coords());
        }

        None
    }

    /// Scaled green coordinates of the cHRM chunk
    pub fn chrm_green_coords(&self) -> Option<(f64, f64)> {
        if let PNGChunkData::CHRM(chrm) = self {
            return Some(chrm.green_coords());
        }

        None
    }

    /// Scaled blue coordinates of the cHRM chunk
    pub fn chrm_blue_coords(&self) -> Option<(f64, f64)> {
        if let PNGChunkData::CHRM(chrm) = self {
            return Some(chrm.blue_coords());
        }

        None
    }

    /// Scaled gamma value of a gAMA chunk
    pub fn gama_gamma(&self) -> Option<f64> {
        if let PNGChunkData::GAMA { gamma } = self {
            return Some(*gamma as f64 / 100000.0);
        }

        None
    }

    /// Decompress the compressed profile in a iCCP chunk
    pub fn iccp_profile(&self) -> Option<Vec<u8>> {
        if let PNGChunkData::ICCP(iccp) = self {
            return iccp.profile();
        } else {
            None
        }
    }

    /// Decompress the compressed string in a zTXt chunk
    pub fn ztxt_string(&self) -> Option<String> {
        if let PNGChunkData::ZTXT(ztxt) = self {
            return ztxt.string();
        }

        None
    }

    /// Decompress the compressed string in an iTXt chunk
    pub fn itxt_string(&self) -> Option<String> {
        if let PNGChunkData::ITXT(itxt) = self {
            return itxt.string();
        }

        None
    }

    /// Convert the units in a pHYs chunk to a UoM type
    pub fn phys_res(&self) -> Option<(LinearNumberDensity, LinearNumberDensity)> {
        if let PNGChunkData::PHYS {
            x_pixels_per_unit,
            y_pixels_per_unit,
            unit,
        } = self
        {
            return match unit {
                PNGUnitType::Unknown => None,

                PNGUnitType::Metre => Some((
                    LinearNumberDensity::new::<per_meter>(*x_pixels_per_unit as f64),
                    LinearNumberDensity::new::<per_meter>(*y_pixels_per_unit as f64),
                )),
            };
        }

        None
    }

    /// Convert the timestamp in a tIME chunk to a chrono DateTime object
    pub fn time(&self) -> Option<DateTime<Utc>> {
        if let PNGChunkData::TIME {
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
        if let PNGChunkData::FCTL(fctl) = self {
            return Some(fctl.delay());
        }

        None
    }
}

/// Primary chromaticities and white point
///
/// Values are scaled by 100000
#[derive(Clone, Debug, Default)]
pub struct CHRM {
    pub white_x: u32,
    pub white_y: u32,
    pub red_x: u32,
    pub red_y: u32,
    pub green_x: u32,
    pub green_y: u32,
    pub blue_x: u32,
    pub blue_y: u32,
}

impl CHRM {
    /// Scaled white coordinates of the cHRM chunk
    pub fn white_coords(&self) -> (f64, f64) {
        (
            self.white_x as f64 / 100000.0,
            self.white_y as f64 / 100000.0,
        )
    }

    /// Scaled red coordinates of the cHRM chunk
    pub fn red_coords(&self) -> (f64, f64) {
        (self.red_x as f64 / 100000.0, self.red_y as f64 / 100000.0)
    }

    /// Scaled green coordinates of the cHRM chunk
    pub fn green_coords(&self) -> (f64, f64) {
        (
            self.green_x as f64 / 100000.0,
            self.green_y as f64 / 100000.0,
        )
    }

    /// Scaled blue coordinates of the cHRM chunk
    pub fn blue_coords(&self) -> (f64, f64) {
        (self.blue_x as f64 / 100000.0, self.blue_y as f64 / 100000.0)
    }
}

/// Mastering Display Color Volume
#[derive(Clone, Copy, Debug, Default)]
pub struct MDCV {
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

impl MDCV {
    /// Scaled red coordinates of the mDCV chunk
    pub fn red_coords(&self) -> (f64, f64) {
        (self.red_x as f64 / 50000.0, self.red_y as f64 / 50000.0)
    }

    /// Scaled green coordinates of the mDCV chunk
    pub fn green_coords(&self) -> (f64, f64) {
        (self.green_x as f64 / 50000.0, self.green_y as f64 / 50000.0)
    }

    /// Scaled blue coordinates of the mDCV chunk
    pub fn blue_coords(&self) -> (f64, f64) {
        (self.blue_x as f64 / 50000.0, self.blue_y as f64 / 50000.0)
    }

    /// Scaled white coordinates of the mDCV chunk
    pub fn white_coords(&self) -> (f64, f64) {
        (self.white_x as f64 / 50000.0, self.white_y as f64 / 50000.0)
    }

    /// Scaled mastering display maximum luminance of the mDCV chunk
    pub fn max_lum(&self) -> Luminance {
        Luminance::new::<candela_per_square_meter>(self.max_lum as f64 / 10000.0)
    }

    /// Scaled mastering display minimum luminance of the mDCV chunk
    pub fn min_lum(&self) -> Luminance {
        Luminance::new::<candela_per_square_meter>(self.min_lum as f64 / 10000.0)
    }
}

/// Content Light Level Information
#[derive(Clone, Copy, Debug, Default)]
pub struct CLLI {
    /// Maximum Content Light Level
    pub max_cll: u32,

    /// Maximum Frame-Average Light Level
    pub max_fall: u32,
}

impl CLLI {
    /// Scaled maximum content light level
    pub fn max_cll(&self) -> Luminance {
        Luminance::new::<candela_per_square_meter>(self.max_cll as f64 / 10000.0)
    }

    /// Scaled maximum frame-average Light Level
    pub fn max_fall(&self) -> Luminance {
        Luminance::new::<candela_per_square_meter>(self.max_fall as f64 / 10000.0)
    }
}

/// Embedded ICC profile
#[derive(Clone, Debug, Default)]
pub struct ICCP {
    pub name: String,
    pub compression_method: PNGCompressionMethod,
    pub compressed_profile: Vec<u8>,
}

impl ICCP {
    pub fn profile(&self) -> Option<Vec<u8>> {
        if self.compression_method == PNGCompressionMethod::Zlib {
            inflate_bytes_zlib(self.compressed_profile.as_slice()).ok()
        } else {
            None
        }
    }
}

/// Textual data
#[derive(Clone, Debug)]
pub struct TEXT {
    pub keyword: String,
    pub string: String,
}

/// Compressed textual data
#[derive(Clone, Debug, Default)]
pub struct ZTXT {
    pub keyword: String,
    pub compression_method: PNGCompressionMethod,
    pub compressed_string: Vec<u8>,
}

impl ZTXT {
    /// Decompress the compressed string in a zTXt chunk
    pub fn string(&self) -> Option<String> {
        if self.compression_method == PNGCompressionMethod::Zlib {
            let bytes = inflate_bytes_zlib(self.compressed_string.as_slice()).ok()?;
            return String::from_utf8(bytes).ok();
        }

        None
    }
}

/// International textual data
#[derive(Clone, Debug, Default)]
pub struct ITXT {
    pub keyword: String,
    pub compressed: bool,
    pub compression_method: PNGCompressionMethod,
    pub language: String,
    pub translated_keyword: String,
    pub compressed_string: Vec<u8>,
}

impl ITXT {
    /// Decompress the compressed string in an iTXt chunk
    pub fn string(&self) -> Option<String> {
        if self.compressed {
            if self.compression_method == PNGCompressionMethod::Zlib {
                let bytes = inflate_bytes_zlib(self.compressed_string.as_slice()).ok()?;
                String::from_utf8(bytes).ok()
            } else {
                None
            }
        } else {
            String::from_utf8(self.compressed_string.to_vec()).ok()
        }
    }
}

/// Suggested palette
#[derive(Clone, Debug, Default)]
pub struct SPLT {
    pub name: String,
    pub depth: u8,
    pub palette: Vec<PNGSuggestedPaletteEntry>,
}

/// Calibration of pixel values
#[derive(Clone, Debug, Default)]
pub struct PCAL {
    pub name: String,
    pub original_zero: u32,
    pub original_max: u32,
    pub equation_type: u8,
    pub unit_name: String,
    pub parameters: Vec<String>,
}

/// Physical scale of image subject
#[derive(Clone, Debug)]
pub struct SCAL {
    pub unit: PNGUnitType,
    pub pixel_width: String,
    pub pixel_height: String,
}

/// GIF Application Extension
#[derive(Clone, Debug)]
pub struct GIFX {
    pub app_id: String,
    pub app_code: [u8; 3],
    pub app_data: Vec<u8>,
}

/// Frame control
#[derive(Clone, Debug)]
pub struct FCTL {
    pub sequence_number: u32,
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub delay_num: u16,
    pub delay_den: u16,
    pub dispose_op: APNGDisposalOperator,
    pub blend_op: APNGBlendOperator,
}

impl FCTL {
    /// Calculate delay from fcTL chunk in seconds
    pub fn delay(&self) -> Time {
        Time::new::<uom::si::time::second>(self.delay_num as f64 / self.delay_den as f64)
    }
}

/// Frame data
#[derive(Clone, Debug)]
pub struct FDAT {
    pub sequence_number: u32,
    pub frame_data: Vec<u8>,
}

/// Reference to a chunk in a PNG file
#[derive(Copy, Clone, Debug, Default)]
pub struct PNGChunkRef {
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

impl PNGChunkRef {
    /// Read the length and type of a chunk from a [Read]'able stream to make a chunk reference
    ///
    /// This leaves the stream at the start of chunk data.
    pub fn new<R>(stream: &mut R, position: u64) -> Result<Self, std::io::Error>
    where
        R: Read,
    {
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
    pub fn read_fctl_fdat_sequence_number<R>(&self, stream: &mut R) -> Result<u32, std::io::Error>
    where
        R: Read,
    {
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

    /// Read the chunk data and parse it into a PNGChunkData enum
    ///
    /// Stream must be at the start of chunk data after the length and type fields.
    /// This also checks the chunk CRC value.
    pub fn read_chunk<R>(
        &self,
        stream: &mut R,
        ihdr: Option<&PNGChunkData>,
    ) -> Result<PNGChunkData, std::io::Error>
    where
        R: Read,
    {
        let mut chunkstream = stream.take(self.length as u64);

        let mut data_crc = CRC::new();
        data_crc.consume(&self.chunktype);

        let chunk = match &self.chunktype {
            b"IHDR" => {
                let mut buf = [ 0_u8; 13 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::IHDR {
                    width: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    height: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                    bit_depth: buf[8],
                    colour_type: buf[9].try_into().map_err(to_io_error)?,
                    compression_method: buf[10].try_into().map_err(to_io_error)?,
                    filter_method: buf[11].try_into().map_err(to_io_error)?,
                    interlace_method: buf[12].try_into().map_err(to_io_error)?,
                })
            }

            b"PLTE" => Ok(PNGChunkData::PLTE(Box::new(
                (0..self.length / 3)
                    .map(|_| {
                        let mut buf = [0_u8; 3];
                        chunkstream.read_exact(&mut buf)?;
                        data_crc.consume(&buf);
                        Ok(PNGPaletteEntry {
                            red: buf[0],
                            green: buf[1],
                            blue: buf[2],
                        })
                    })
                    .collect::<Result<Vec<_>, std::io::Error>>()?,
            ))),

            b"IDAT" => {
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::IDAT(Box::new(data)))
            }

            b"IEND" => Ok(PNGChunkData::IEND),

            b"tRNS" => {
                if ihdr.is_none() {
                    return Err(std::io::Error::other("PNG: Unset ihdr".to_string()));
                }

                if let PNGChunkData::IHDR { colour_type, .. } = ihdr.unwrap() {
                    match *colour_type {
                        PNGColourType::Greyscale => {
                            let mut buf = [0_u8; 2];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::TRNS {
                                data: PNGtRNSType::Greyscale {
                                    value: u16::from_be_bytes(buf),
                                },
                            })
                        }

                        PNGColourType::TrueColour => {
                            let mut buf = [0_u8; 6];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::TRNS {
                                data: PNGtRNSType::TrueColour {
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

                        PNGColourType::IndexedColour => {
                            let mut values = vec![ 0_u8; self.length as usize ];
                            chunkstream.read_exact(&mut values)?;
                            data_crc.consume(&values);

                            Ok(PNGChunkData::TRNS {
                                data: PNGtRNSType::IndexedColour { values },
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

                Ok(PNGChunkData::GAMA {
                    gamma: u32::from_be_bytes(buf),
                })
            }

            b"cHRM" => {
                let mut data = [ 0_u8; 32 ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::CHRM(Box::new(CHRM {
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
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let name_end = find_null(&data);
                Ok(PNGChunkData::ICCP(Box::new(ICCP {
                    name: String::from_utf8(data[0..name_end].to_vec()).map_err(to_io_error)?,
                    compression_method: data[name_end].try_into().map_err(to_io_error)?,
                    compressed_profile: data[name_end + 2..].to_vec(),
                })))
            }

            b"sBIT" => {
                if ihdr.is_none() {
                    return Err(std::io::Error::other("PNG: Unset ihdr".to_string()));
                }

                if let PNGChunkData::IHDR { colour_type, .. } = ihdr.unwrap() {
                    match colour_type {
                        PNGColourType::Greyscale => {
                            let mut buf = [0_u8; 1];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::SBIT {
                                bits: PNGsBITType::Greyscale { grey_bits: buf[0] },
                            })
                        }

                        PNGColourType::TrueColour | PNGColourType::IndexedColour => {
                            let mut buf = [0_u8; 3];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::SBIT {
                                bits: PNGsBITType::Colour {
                                    red_bits: buf[0],
                                    green_bits: buf[1],
                                    blue_bits: buf[2],
                                },
                            })
                        }

                        PNGColourType::GreyscaleAlpha => {
                            let mut buf = [0_u8; 2];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::SBIT {
                                bits: PNGsBITType::GreyscaleAlpha {
                                    grey_bits: buf[0],
                                    alpha_bits: buf[1],
                                },
                            })
                        }

                        PNGColourType::TrueColourAlpha => {
                            let mut buf = [0_u8; 4];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::SBIT {
                                bits: PNGsBITType::TrueColourAlpha {
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

                Ok(PNGChunkData::SRGB {
                    rendering_intent: buf[0].try_into().map_err(to_io_error)?,
                })
            }

            b"cICP" => {
                let mut buf = [0_u8; 4];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::CICP {
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

                Ok(PNGChunkData::MDCV(Box::new(MDCV {
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

                Ok(PNGChunkData::CLLI(Box::new(CLLI {
                    max_cll: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    max_fall: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                })))
            }

            b"tEXt" => {
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                Ok(PNGChunkData::TEXT(Box::new(TEXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .map_err(to_io_error)?,
                    string: String::from_utf8(data[keyword_end + 1..].to_vec())
                        .map_err(to_io_error)?,
                })))
            }

            b"zTXt" => {
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                Ok(PNGChunkData::ZTXT(Box::new(ZTXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .map_err(to_io_error)?,
                    compression_method: data[keyword_end + 1].try_into().map_err(to_io_error)?,
                    compressed_string: data[keyword_end + 2..].to_vec(),
                })))
            }

            b"iTXt" => {
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                let language_end = find_null(&data[keyword_end + 3..]) + keyword_end + 3;
                let tkeyword_end = find_null(&data[language_end + 1..]) + language_end + 1;

                Ok(PNGChunkData::ITXT(Box::new(ITXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .map_err(to_io_error)?,
                    compressed: data[keyword_end + 1] > 0,
                    compression_method: data[keyword_end + 2].try_into().map_err(to_io_error)?,
                    language: String::from_utf8(data[keyword_end + 3..language_end].to_vec())
                        .map_err(to_io_error)?,
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

                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                if let PNGChunkData::IHDR { colour_type, .. } = ihdr.unwrap() {
                    match colour_type {
                        PNGColourType::Greyscale | PNGColourType::GreyscaleAlpha => {
                            if self.length != 2 {
                                return Err(std::io::Error::other(format!(
                                    "PNG: Invalid length of bKGD chunk ({})",
                                    self.length
                                )));
                            }

                            Ok(PNGChunkData::BKGD {
                                data: PNGbKGDType::Greyscale {
                                    value: u16::from_be_bytes(
                                        data[0..2].try_into().map_err(to_io_error)?,
                                    ),
                                },
                            })
                        }

                        PNGColourType::TrueColour | PNGColourType::TrueColourAlpha => {
                            if self.length != 6 {
                                return Err(std::io::Error::other(format!(
                                    "PNG: Invalid length of bKGD chunk ({})",
                                    self.length
                                )));
                            }

                            Ok(PNGChunkData::BKGD {
                                data: PNGbKGDType::TrueColour {
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

                        PNGColourType::IndexedColour => {
                            if self.length != 1 {
                                return Err(std::io::Error::other(format!(
                                    "PNG: Invalid length of bKGD chunk ({})",
                                    self.length
                                )));
                            }

                            Ok(PNGChunkData::BKGD {
                                data: PNGbKGDType::IndexedColour { index: data[0] },
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
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::HIST(Box::new(
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

                Ok(PNGChunkData::PHYS {
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
                let mut profile = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut profile)?;
                data_crc.consume(&profile);

                Ok(PNGChunkData::EXIF(Box::new(profile)))
            }

            b"sPLT" => {
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let name_end = find_null(&data);
                let depth = data[name_end + 1];
                let entry_size = ((depth / 8) * 4) + 2;
                let num_entries = (self.length as usize - name_end - 1) / (entry_size as usize);

                Ok(PNGChunkData::SPLT(Box::new(SPLT {
                    name: String::from_utf8(data[0..name_end].to_vec()).map_err(to_io_error)?,
                    depth,
                    palette: (0..num_entries)
                        .map(|i| {
                            let start = name_end + 2 + (i * entry_size as usize);
                            if depth == 8 {
                                Ok(PNGSuggestedPaletteEntry {
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
                                Ok(PNGSuggestedPaletteEntry {
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

                Ok(PNGChunkData::TIME {
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

                Ok(PNGChunkData::ACTL {
                    num_frames: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    num_plays: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                })
            }

            b"fcTL" => {
                let mut buf = [0_u8; 26];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::FCTL(Box::new(FCTL {
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
                let mut buf = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::FDAT(Box::new(FDAT {
                    sequence_number: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    frame_data: buf[4..].to_vec(),
                })))
            }

            // Extensions
            b"oFFs" => {
                let mut buf = [0_u8; 9];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::OFFS {
                    x: u32::from_be_bytes(buf[0..4].try_into().map_err(to_io_error)?),
                    y: u32::from_be_bytes(buf[4..8].try_into().map_err(to_io_error)?),
                    unit: buf[8].try_into().map_err(to_io_error)?,
                })
            }

            b"pCAL" => {
                let mut data = vec![ 0_u8; self.length as usize ];
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
                        String::from_utf8(data[prev_end..param_end].to_vec())
                            .map_err(to_io_error)?,
                    );
                    prev_end = param_end;
                }

                Ok(PNGChunkData::PCAL(Box::new(PCAL {
                    name: String::from_utf8(data[0..name_end].to_vec()).map_err(to_io_error)?,
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
                    unit_name: String::from_utf8(data[name_end + 10..unit_end].to_vec())
                        .map_err(to_io_error)?,
                    parameters,
                })))
            }

            b"sCAL" => {
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                let width_end = find_null(&data[1..]) + 1;
                let height_end = find_null(&data[width_end..]) + width_end;

                Ok(PNGChunkData::SCAL(Box::new(SCAL {
                    unit: data[0].try_into().map_err(to_io_error)?,
                    pixel_width: String::from_utf8(data[1..width_end].to_vec())
                        .map_err(to_io_error)?,
                    pixel_height: String::from_utf8(data[width_end..height_end].to_vec())
                        .map_err(to_io_error)?,
                })))
            }

            b"gIFg" => {
                let mut buf = [0_u8; 4];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::GIFG {
                    disposal_method: buf[0],
                    user_input: buf[1] > 0,
                    delay_time: u16::from_be_bytes(buf[2..].try_into().map_err(to_io_error)?),
                })
            }

            b"gIFx" => {
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::GIFX(Box::new(GIFX {
                    app_id: String::from_utf8(data[0..8].to_vec()).map_err(to_io_error)?,
                    app_code: [data[8], data[9], data[10]],
                    app_data: data[11..].to_vec(),
                })))
            }

            b"sTER" => {
                let mut buf = [0_u8; 1];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::STER { mode: buf[0] })
            }

            // JNG chunks
            b"JHDR" => {
                let mut buf = [0_u8; 16];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::JHDR {
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
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::JDAT(Box::new(data)))
            }

            b"JDAA" => {
                let mut data = vec![ 0_u8; self.length as usize ];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::JDAA(Box::new(data)))
            }

            b"JSEP" => Ok(PNGChunkData::JSEP),

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
pub struct APNGFrame {
    /// The fcTL chunk defining the frame
    pub fctl: PNGChunkRef,

    /// The fdAT chunk(s) containing the frame data
    pub fdats: Vec<PNGChunkRef>,
}

/// An iterator for reading IDAT/fdAT/JDAT/JDAA chunks from a PNG/APNG/JNG image
pub struct PNGDATChunkIter<'a, R> {
    stream: &'a mut R,

    position: u64,
}

impl<'a, R> PNGDATChunkIter<'a, R>
where
    R: Read,
{
    /// Constructor
    ///
    /// `stream`: anything that implements [Read].\
    /// `position`: The current stream position, at the start of a chunk.
    pub fn new(stream: &'a mut R, position: u64) -> Self {
        Self { stream, position }
    }
}

impl<'a, R> Iterator for PNGDATChunkIter<'a, R>
where
    R: Read,
{
    type Item = PNGChunkData;

    /// Get the next IDAT/fdAT/JDAT/JDAA chunk
    fn next(&mut self) -> Option<Self::Item> {
        let mut chunkref = PNGChunkRef::new(self.stream, self.position).ok()?;
        if chunkref.chunktype == *b"IEND" {
            return None;
        }

        eprintln!(
            "PNGDATChunkIter: Read {} chunk ref, reading {} bytes of data...",
            chunkref.type_str(),
            chunkref.length
        );
        let mut chunk = chunkref.read_chunk(self.stream, None).ok()?;
        self.position += 4 + 4 + chunkref.length as u64 + 4;

        while chunkref.chunktype != *b"IDAT"
            && chunkref.chunktype != *b"fdAT"
            && chunkref.chunktype != *b"JDAT"
            && chunkref.chunktype != *b"JDAA"
        {
            chunkref = PNGChunkRef::new(self.stream, self.position).ok()?;
            if chunkref.chunktype == *b"IEND" {
                return None;
            }

            eprintln!(
                "PNGDATChunkIter: Read {} chunk ref, reading {} bytes of data...",
                chunkref.type_str(),
                chunkref.length
            );
            chunk = chunkref.read_chunk(self.stream, None).ok()?;
            self.position += 4 + 4 + chunkref.length as u64 + 4;
        }

        Some(chunk)
    }
}
