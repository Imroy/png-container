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
use std::str;

use chrono::{DateTime, NaiveDate, NaiveTime, NaiveDateTime, Utc};
use inflate::inflate_bytes_zlib;
use uom::si::{
    f64::{LinearNumberDensity, Time},
    linear_number_density::per_meter,
};

use crate::to_io_error;
use crate::crc::*;
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
        compression_method: PNGCompressionMethod,
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
        compression_method: PNGCompressionMethod,
        compressed_string: Vec<u8>,
    },

    /// International textual data
    ITXT {
        keyword: String,
        compressed: bool,
        compression_method: PNGCompressionMethod,
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
        palette: Vec<PNGSuggestedPaletteEntry>,
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
        // TODO: make this an enum
        disposal_method: u8,
        user_input: bool,
        delay_time: u16,
    },

    /// GIF Application Extension
    GIFX {
        app_id: String,
        app_code: [ u8; 3 ],
        app_data: Vec<u8>,
    },

    /// Indicator of Stereo Image
    STER {
        mode: u8,
    },

    // Animation information

    /// Animation control
    ACTL {
        num_frames: u32,
        num_plays: u32,
    },

    /// Frame control
    FCTL {
        sequence_number: u32,
        width: u32,
        height: u32,
        x_offset: u32,
        y_offset: u32,
        delay_num: u16,
        delay_den: u16,
        dispose_op: APNGDisposalOperator,
        blend_op: APNGBlendOperator,
    },

    FDAT {
        sequence_number: u32,
        frame_data: Vec<u8>,
    },

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
    JDAT {
        data: Vec<u8>,
    },

    /// JNG alpha data
    JDAA {
        data: Vec<u8>,
    },

    /// JNG image separator
    JSEP,

}

impl PNGChunkData {
    /// Scaled white coordinates of the cHRM chunk
    pub fn chrm_white_coords(&self) -> Option<(f64, f64)> {
        if let PNGChunkData::CHRM { white_x, white_y, .. } = self {
            return Some((*white_x as f64 / 100000.0, *white_y as f64 / 100000.0));
        }

        None
    }

    /// Scaled red coordinates of the cHRM chunk
    pub fn chrm_red_coords(&self) -> Option<(f64, f64)> {
        if let PNGChunkData::CHRM { red_x, red_y, .. } = self {
            return Some((*red_x as f64 / 100000.0, *red_y as f64 / 100000.0));
        }

        None
    }

    /// Scaled green coordinates of the cHRM chunk
    pub fn chrm_green_coords(&self) -> Option<(f64, f64)> {
        if let PNGChunkData::CHRM { green_x, green_y, .. } = self {
            return Some((*green_x as f64 / 100000.0, *green_y as f64 / 100000.0));
        }

        None
    }

    /// Scaled blue coordinates of the cHRM chunk
    pub fn chrm_blue_coords(&self) -> Option<(f64, f64)> {
        if let PNGChunkData::CHRM { blue_x, blue_y, .. } = self {
            return Some((*blue_x as f64 / 100000.0, *blue_y as f64 / 100000.0));
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
        if let PNGChunkData::ICCP { compression_method, compressed_profile, .. } = self {
            if *compression_method == PNGCompressionMethod::Zlib {
                return inflate_bytes_zlib(compressed_profile.as_slice()).ok();
            }
        }

        None
    }

    /// Decompress the compressed string in a zTXt chunk
    pub fn ztxt_string(&self) -> Option<String> {
        if let PNGChunkData::ZTXT { compression_method, compressed_string, .. } = self {
            if *compression_method  == PNGCompressionMethod::Zlib {
                let bytes = inflate_bytes_zlib(compressed_string.as_slice()).ok()?;
                return String::from_utf8(bytes).ok();
            }
        }

        None
    }

    /// Decompress the compressed string in an iTXt chunk
    pub fn itxt_string(&self) -> Option<String> {
        if let PNGChunkData::ITXT { compressed, compression_method,
                                    compressed_string, .. } = self {
            if *compressed {
                if *compression_method == PNGCompressionMethod::Zlib {
                    let bytes = inflate_bytes_zlib(compressed_string.as_slice()).ok()?;
                    return String::from_utf8(bytes).ok();
                }
            } else {
                return String::from_utf8(compressed_string.to_vec()).ok();
            }
        }

        None
    }

    /// Convert the units in a pHYs chunk to a UoM type
    pub fn phys_res(&self) -> Option<(LinearNumberDensity, LinearNumberDensity)> {
        if let PNGChunkData::PHYS { x_pixels_per_unit, y_pixels_per_unit, unit } = self {
            return match unit {
                PNGUnitType::Unknown =>
                    None,

                PNGUnitType::Metre =>
                    Some((LinearNumberDensity::new::<per_meter>(*x_pixels_per_unit as f64),
                       LinearNumberDensity::new::<per_meter>(*y_pixels_per_unit as f64))),
            };
        }

        None
    }

    pub fn time(&self) -> Option<DateTime<Utc>> {
        if let PNGChunkData::TIME { year, month, day, hour, minute, second } = self {
            return Some(DateTime::from_utc(
                NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(*year as i32, *month as u32, *day as u32)?,
                    NaiveTime::from_hms_opt(*hour as u32, *minute as u32, *second as u32)?
                ),
                Utc));
        }

        None
    }

    /// Calculate delay from fcTL chunk in seconds
    pub fn fctl_delay(&self) -> Option<Time> {
        if let PNGChunkData::FCTL { delay_num, delay_den, .. } = self {
            return Some(Time::new::<uom::si::time::second>(*delay_num as f64 / *delay_den as f64));
        }

        None
    }

}


/// Reference to a chunk in a PNG file
#[derive(Copy, Clone, Debug)]
pub struct PNGChunkRef {
    /// The position in the stream/file for this chunk
    pub position: u64,

    /// Length of this chunk
    pub length: u32,

    /// Chunk type
    pub chunktype: [ u8; 4 ],

}

// because u32::from_be_bytes() only takes fixed-length arrays and it's too
// much of a PITA to convert from a slice.
fn u32_be(bytes: &[u8]) -> u32 {
    (bytes[3] as u32) | ((bytes[2] as u32) << 8)
        | ((bytes[1] as u32) << 16) | ((bytes[0] as u32) << 24)
}

fn u16_be(bytes: &[u8]) -> u16 {
    (bytes[1] as u16) | ((bytes[0] as u16) << 8)
}

fn find_null(bytes: &[u8]) -> usize {
    bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len())
}

impl Default for PNGChunkRef {
    fn default() -> Self {
        PNGChunkRef {
            position: 0,
            length: 0,
            chunktype: [ 0_u8; 4 ],
        }
    }

}

impl PNGChunkRef {
    /// Convert the chunk type bytes to a string that can be compared and printed much more easily
    #[inline]
    pub fn type_str(&self) -> &str {
        str::from_utf8(&self.chunktype).unwrap_or("")
    }

    /// Is this chunk necessary for successful display of the contents of
    /// the datastream (false) or not (true)? Derived from the case of the
    /// first character of the chunk type.
    #[inline]
    pub fn is_ancillary(&self) ->  bool {
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

    /// Read the sequence number of an fcTL or fdAT chunk
    pub fn read_fctl_fdat_sequence_number<R>(&self, stream: &mut R)
                                             -> Result<u32, std::io::Error>
    where R: Read
    {
        let mut chunkstream = stream.take(self.length as u64);
        match self.type_str() {
            "fcTL" | "fdAT" => {
                let mut buf4 = [ 0_u8; 4 ];
                chunkstream.read_exact(&mut buf4)?;
                Ok(u32::from_be_bytes(buf4))
            },

            _ => Err(std::io::Error::other(format!(
                "PNG: Chunk type ({}) is not an fcTL or fdAT", self.type_str())))
        }
    }

    /// Read the chunk data and parse it into a PNGChunkData enum
    ///
    /// This also checks the chunk CRC value
    pub fn read_chunk<R>(&self, stream: &mut R,
                         ihdr: Option<&PNGChunkData>)
                         -> Result<PNGChunkData, std::io::Error>
        where R: Read
    {
        let mut chunkstream = stream.take(self.length as u64);

        let mut data_crc = CRC::new();
        data_crc.consume(&self.chunktype);

        let chunk = match self.type_str() {
            "IHDR" => {
                let mut buf = Vec::with_capacity(13);
                chunkstream.read_to_end(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::IHDR {
                    width: u32_be(&buf[0..4]),
                    height: u32_be(&buf[4..8]),
                    bit_depth: buf[8],
                    colour_type: buf[9].try_into()
                        .map_err(to_io_error)?,
                    compression_method: buf[10].try_into()
                        .map_err(to_io_error)?,
                    filter_method: buf[11].try_into()
                        .map_err(to_io_error)?,
                    interlace_method: buf[12].try_into()
                        .map_err(to_io_error)?,
                })
            },

            "PLTE" => {
                let num_entries = self.length / 3;
                let mut entries = Vec::with_capacity(num_entries as usize);
                for _n in 0..num_entries {
                    let mut buf = [ 0_u8; 3 ];
                    chunkstream.read_exact(&mut buf)?;
                    data_crc.consume(&buf);
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
                data_crc.consume(&data);

                Ok(PNGChunkData::IDAT {
                    data,
                })
            }

            "IEND" => Ok(PNGChunkData::IEND),

            "tRNS" => {
                if ihdr.is_none() {
                    return Err(std::io::Error::other(format!("PNG: Unset ihdr")));
                }

                if let PNGChunkData::IHDR { colour_type, .. } = ihdr.unwrap() {
                    match *colour_type {
                        PNGColourType::Greyscale => {
                            let mut buf = [ 0_u8; 2 ];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::TRNS {
                                data: PNGtRNSType::Greyscale {
                                    value: u16_be(&buf),
                                },
                            })
                        },

                        PNGColourType::TrueColour => {
                            let mut buf = [ 0_u8; 6 ];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::TRNS {
                                data: PNGtRNSType::TrueColour {
                                    red: u16_be(&buf[0..2]),
                                    green: u16_be(&buf[2..4]),
                                    blue: u16_be(&buf[4..6]),
                                },
                            })
                        },

                        PNGColourType::IndexedColour => {
                            let mut values = Vec::with_capacity(self.length as usize);
                            chunkstream.read_to_end(&mut values)?;
                            data_crc.consume(&values);

                            Ok(PNGChunkData::TRNS {
                                data: PNGtRNSType::IndexedColour {
                                    values,
                                },
                            })
                        },

                        _ => Err(std::io::Error::other(format!(
                            "PNG: Invalid colour type ({}) in ihdr", *colour_type as u8))),

                    }
                } else {
                    Err(std::io::Error::other("PNG: Wrong chunk type passed as ihdr"))
                }
            },

            "gAMA" => {
                let mut buf = [ 0_u8; 4 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::GAMA {
                    gamma: u32_be(&buf),
                })
            },

            "cHRM" => {
                let mut data = Vec::with_capacity(32);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::CHRM {
                    white_x: u32_be(&data[0..4]),
                    white_y: u32_be(&data[4..8]),
                    red_x: u32_be(&data[8..12]),
                    red_y: u32_be(&data[12..16]),
                    green_x: u32_be(&data[16..20]),
                    green_y: u32_be(&data[20..24]),
                    blue_x: u32_be(&data[24..28]),
                    blue_y: u32_be(&data[28..32]),
                })
            },

            "iCCP" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                let name_end = find_null(&data);
                Ok(PNGChunkData::ICCP {
                    name: String::from_utf8(data[0..name_end].to_vec())
                        .map_err(to_io_error)?,
                    compression_method: data[name_end].try_into()
                        .map_err(to_io_error)?,
                    compressed_profile: data[name_end + 2..].to_vec(),
                })
            },

            "sBIT" => {
                if ihdr.is_none() {
                    return Err(std::io::Error::other(format!("PNG: Unset ihdr")));
                }

                if let PNGChunkData::IHDR { colour_type, .. } = ihdr.unwrap() {
                    match colour_type {
                        PNGColourType::Greyscale => {
                            let mut buf = [ 0_u8; 1 ];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::SBIT {
                                bits: PNGsBITType::Greyscale {
                                    grey_bits: buf[0],
                                },
                            })
                        },

                        PNGColourType::TrueColour
                            | PNGColourType::IndexedColour =>
                        {
                            let mut buf = [ 0_u8; 3 ];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::SBIT {
                                bits: PNGsBITType::Colour {
                                    red_bits: buf[0],
                                    green_bits: buf[1],
                                    blue_bits: buf[2],
                                },
                            })
                        },

                        PNGColourType::GreyscaleAlpha => {
                            let mut buf = [ 0_u8; 2 ];
                            chunkstream.read_exact(&mut buf)?;
                            data_crc.consume(&buf);

                            Ok(PNGChunkData::SBIT {
                                bits: PNGsBITType::GreyscaleAlpha {
                                    grey_bits: buf[0],
                                    alpha_bits: buf[1],
                                },
                            })
                        },

                        PNGColourType::TrueColourAlpha => {
                            let mut buf = [ 0_u8; 4 ];
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
                        },

                    }
                } else {
                    Err(std::io::Error::other("PNG: Wrong chunk type passed as ihdr"))
                }
            },

            "sRGB" => {
                let mut buf = [ 0_u8; 1 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::SRGB {
                    rendering_intent: buf[0].try_into()
                        .map_err(to_io_error)?,
                })
            },

            "cICP" => {
                let mut buf = [ 0_u8; 4 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::CICP {
                    colour_primaries: buf[0],
                    transfer_function: buf[1],
                    matrix_coeffs: buf[2],
                    video_full_range: buf[3] > 0,
                })
            }

            "tEXt" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                Ok(PNGChunkData::TEXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .map_err(to_io_error)?,
                    string: String::from_utf8(data[keyword_end + 1..].to_vec())
                        .map_err(to_io_error)?,
                })
            },

            "zTXt" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                Ok(PNGChunkData::ZTXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .map_err(to_io_error)?,
                    compression_method: data[keyword_end + 1].try_into()
                        .map_err(to_io_error)?,
                    compressed_string: data[keyword_end + 2..].to_vec(),
                })
            },

            "iTXt" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                let keyword_end = find_null(&data);
                let language_end = find_null(&data[keyword_end + 3..])
                    + keyword_end + 3;
                let tkeyword_end = find_null(&data[language_end + 1..])
                    + language_end + 1;

                Ok(PNGChunkData::ITXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .map_err(to_io_error)?,
                    compressed: data[keyword_end + 1] > 0,
                    compression_method: data[keyword_end + 2].try_into()
                        .map_err(to_io_error)?,
                    language: String::from_utf8(data[keyword_end + 3..language_end]
                                                .to_vec())
                        .map_err(to_io_error)?,
                    translated_keyword: String::from_utf8(data[language_end + 1..tkeyword_end]
                                                          .to_vec())
                        .map_err(to_io_error)?,
                    compressed_string: data[tkeyword_end + 1..].to_vec(),
                })
            },

            "bKGD" => {
                if ihdr.is_none() {
                    return Err(std::io::Error::other(format!("PNG: Unset ihdr")));
                }

                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                if let PNGChunkData::IHDR { colour_type, .. } = ihdr.unwrap() {
                    match colour_type {
                        PNGColourType::Greyscale | PNGColourType::GreyscaleAlpha => {
                            if self.length != 2 {
                                return Err(std::io::Error::other(format!(
                                    "PNG: Invalid length of bKGD chunk ({})",
                                    self.length)));
                            }

                            Ok(PNGChunkData::BKGD{
                                data: PNGbKGDType::Greyscale {
                                    value: u16_be(&data[0..2]),
                                },
                            })
                        },

                        PNGColourType::TrueColour
                            | PNGColourType::TrueColourAlpha =>
                        {
                            if self.length != 6 {
                                return Err(std::io::Error::other(format!(
                                    "PNG: Invalid length of bKGD chunk ({})",
                                    self.length)));
                            }

                            Ok(PNGChunkData::BKGD{
                                data: PNGbKGDType::TrueColour {
                                    red: u16_be(&data[0..2]),
                                    green: u16_be(&data[2..4]),
                                    blue: u16_be(&data[4..6]),
                                }
                            })
                        },

                        PNGColourType::IndexedColour => {
                            if self.length != 1 {
                                return Err(std::io::Error::other(format!(
                                    "PNG: Invalid length of bKGD chunk ({})",
                                    self.length)));
                            }

                            Ok(PNGChunkData::BKGD{
                                data: PNGbKGDType::IndexedColour {
                                    index: data[0],
                                }
                            })
                        },
                    }
                } else {
                    Err(std::io::Error::other("PNG: Wrong chunk type passed as ihdr"))
                }
            },

            "hIST" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                let num_entries = self.length / 2;
                let mut frequencies = Vec::with_capacity(num_entries as usize);

                for n in 0..num_entries {
                    let start = n as usize * 2;
                    frequencies.push(u16_be(&data[start..start + 2]));
                }

                Ok(PNGChunkData::HIST {
                    frequencies,
                })
            },

            "pHYs" => {
                let mut buf = [ 0_u8; 9 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::PHYS {
                    x_pixels_per_unit: u32_be(&buf[0..4]),
                    y_pixels_per_unit: u32_be(&buf[4..8]),
                    unit: buf[8].try_into()
                        .map_err(to_io_error)?,
                })
            },

            "eXIf" => {
                let mut profile = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut profile)?;
                data_crc.consume(&profile);

                Ok(PNGChunkData::EXIF {
                    profile,
                })
            },

            "sPLT" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                let name_end = find_null(&data);
                let depth = data[name_end + 1];
                let entry_size = ((depth / 8) * 4) + 2;
                let num_entries = (self.length as usize - name_end - 1)
                    / (entry_size as usize);
                let mut palette = Vec::with_capacity(num_entries);

                for i in 0..num_entries {
                    let start = name_end + 2 + (i * entry_size as usize);
                    if depth == 8 {
                        palette.push(PNGSuggestedPaletteEntry {
                            red: data[start] as u16,
                            green: data[start + 1] as u16,
                            blue: data[start + 2] as u16,
                            alpha: data[start + 3] as u16,
                            frequency: u16_be(&data[start + 4..start + 6]),
                        });
                    } else {
                        palette.push(PNGSuggestedPaletteEntry {
                            red: u16_be(&data[start..start + 2]),
                            green: u16_be(&data[start + 2..start + 4]),
                            blue: u16_be(&data[start + 4..start + 6]),
                            alpha: u16_be(&data[start + 6..start + 8]),
                            frequency: u16_be(&data[start + 8..start + 10]),
                        });
                    }
                }

                Ok(PNGChunkData::SPLT {
                    name: String::from_utf8(data[0..name_end].to_vec())
                        .map_err(to_io_error)?,
                    depth,
                    palette,
                })
            },

            "tIME" => {
                let mut buf = [ 0_u8; 7 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::TIME {
                    year: u16_be(&buf[0..2]),
                    month: buf[2],
                    day: buf[3],
                    hour: buf[4],
                    minute: buf[5],
                    second: buf[6],
                })
            },

            "acTL" => {
                let mut buf = [ 0_u8; 8 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::ACTL {
                    num_frames: u32_be(&buf[0..4]),
                    num_plays: u32_be(&buf[4..8]),
                })
            },

            "fcTL" => {
                let mut buf = [ 0_u8; 26 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::FCTL {
                    sequence_number: u32_be(&buf[0..4]),
                    width: u32_be(&buf[4..8]),
                    height: u32_be(&buf[8..12]),
                    x_offset: u32_be(&buf[12..16]),
                    y_offset: u32_be(&buf[16..20]),
                    delay_num: u16_be(&buf[20..22]),
                    delay_den: u16_be(&buf[22..24]),
                    dispose_op: buf[24].try_into()
                        .map_err(to_io_error)?,
                    blend_op: buf[24].try_into()
                        .map_err(to_io_error)?,
                })
            },

            "fdAT" => {
                let mut buf = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::FDAT {
                    sequence_number: u32_be(&buf[0..4]),
                    frame_data: buf[4..].to_vec(),
                })
            },

            // Extensions

            "oFFs" => {
                let mut buf = [ 0_u8; 9 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::OFFS {
                    x: u32_be(&buf[0..4]),
                    y: u32_be(&buf[4..8]),
                    unit: buf[8].try_into()
                        .map_err(to_io_error)?,
                })
            },

            "pCAL" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                let name_end = find_null(&data);
                let num_parameters = data[name_end + 9];
                let unit_end = find_null(&data[name_end + 10..]) + name_end + 10;
                let mut parameters = Vec::with_capacity(num_parameters as usize);

                let mut prev_end = unit_end;
                for _ in 0..num_parameters {
                    let param_end = find_null(&data[prev_end..]) + prev_end;
                    parameters.push(String::from_utf8(data[prev_end..param_end].to_vec())
                                    .map_err(to_io_error)?);
                    prev_end = param_end;
                }

                Ok(PNGChunkData::PCAL {
                    name: String::from_utf8(data[0..name_end].to_vec())
                        .map_err(to_io_error)?,
                    original_zero: u32_be(&data[name_end..name_end + 4]),
                    original_max: u32_be(&data[name_end + 4..name_end + 8]),
                    equation_type: data[name_end + 8],
                    unit_name: String::from_utf8(data[name_end + 10..unit_end].to_vec())
                        .map_err(to_io_error)?,
                    parameters,
                })
            },

            "sCAL" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                let width_end = find_null(&data[1..]) + 1;
                let height_end = find_null(&data[width_end..]) + width_end;

                Ok(PNGChunkData::SCAL {
                    unit: data[0].try_into()
                        .map_err(to_io_error)?,
                    pixel_width: String::from_utf8(data[1..width_end].to_vec())
                        .map_err(to_io_error)?,
                    pixel_height: String::from_utf8(data[width_end..height_end].to_vec())
                        .map_err(to_io_error)?,
                })
            },

            "gIFg" => {
                let mut buf = [ 0_u8; 4 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::GIFG {
                    disposal_method: buf[0],
                    user_input: buf[1] > 0,
                    delay_time: u16_be(&buf[2..]),
                })
            },

            "gIFx" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::GIFX {
                    app_id: String::from_utf8(data[0..8].to_vec())
                        .map_err(to_io_error)?,
                    app_code: [ data[8], data[9], data[10] ],
                    app_data: data[11..].to_vec(),
                })
            },

            "sTER" => {
                let mut buf = [ 0_u8; 1 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::STER {
                    mode: buf[0],
                })
            },

            "JHDR" => {
                let mut buf = [ 0_u8; 16 ];
                chunkstream.read_exact(&mut buf)?;
                data_crc.consume(&buf);

                Ok(PNGChunkData::JHDR {
                    width: u32_be(&buf[0..4]),
                    height: u32_be(&buf[4..8]),
                    colour_type: buf[8].try_into()
                        .map_err(to_io_error)?,
                    image_sample_depth: buf[9].try_into()
                        .map_err(to_io_error)?,
                    image_compression_method: buf[10].try_into()
                        .map_err(to_io_error)?,
                    image_interlace_method: buf[11].try_into()
                        .map_err(to_io_error)?,
                    alpha_sample_depth: buf[12].try_into()
                        .map_err(to_io_error)?,
                    alpha_compression_method: buf[13].try_into()
                        .map_err(to_io_error)?,
                    alpha_filter_method: buf[14].try_into()
                        .map_err(to_io_error)?,
                    alpha_interlace_method: buf[15].try_into()
                        .map_err(to_io_error)?,
                })
            },

            "JDAT" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::JDAT {
                    data,
                })
            },

            "JDAA" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                data_crc.consume(&data);

                Ok(PNGChunkData::JDAA {
                    data,
                })
            },

            "JSEP" => Ok(PNGChunkData::JSEP),

            _ => Err(std::io::Error::other(format!(
                "PNG: Unhandled chunk type ({})", self.type_str())))
        }?;

        let mut buf4 = [ 0_u8; 4 ];
        stream.read_exact(&mut buf4)?;
        let crc = u32::from_be_bytes(buf4);
        if crc != data_crc.value() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                                           format!("PNG: Read CRC ({:#x}) doesn't match the computed one ({:#x})",
                                                   crc, data_crc.value())));
        }

        Ok(chunk)
    }

}
