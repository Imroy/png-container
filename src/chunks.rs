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
use std::collections::VecDeque;
use std::slice::Iter;
use std::str;
use inflate::inflate_bytes_zlib;

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
        image_interlace_method: JNGInterlaceType,

        /// Alpha sample depth
        alpha_sample_depth: JNGAlphaSampleDepth,

        /// Alpha compression method
        alpha_compression_method: JNGCompressionType,

        /// Alpha channel filter method
        alpha_filter_method: PNGFilterType,

        /// Alpha interlace method
        alpha_interlace_method: JNGInterlaceType,
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
    pub fn chrm_white_coords(&self) -> Result<(f64, f64), String> {
        match self {
            PNGChunkData::CHRM { white_x, white_y, .. } => {
                Ok((*white_x as f64 / 100000.0, *white_y as f64 / 100000.0))
            },

            _ => Err("PNG: Not a cHRM chunk".to_string()),
        }
    }

    /// Scaled red coordinates of the cHRM chunk
    pub fn chrm_red_coords(&self) -> Result<(f64, f64), String> {
        match self {
            PNGChunkData::CHRM { red_x, red_y, .. } => {
                Ok((*red_x as f64 / 100000.0, *red_y as f64 / 100000.0))
            },

            _ => Err("PNG: Not a cHRM chunk".to_string()),
        }
    }

    /// Scaled green coordinates of the cHRM chunk
    pub fn chrm_green_coords(&self) -> Result<(f64, f64), String> {
        match self {
            PNGChunkData::CHRM { green_x, green_y, .. } => {
                Ok((*green_x as f64 / 100000.0, *green_y as f64 / 100000.0))
            },

            _ => Err("PNG: Not a cHRM chunk".to_string()),
        }
    }

    /// Scaled blue coordinates of the cHRM chunk
    pub fn chrm_blue_coords(&self) -> Result<(f64, f64), String> {
        match self {
            PNGChunkData::CHRM { blue_x, blue_y, .. } => {
                Ok((*blue_x as f64 / 100000.0, *blue_y as f64 / 100000.0))
            },

            _ => Err("PNG: Not a cHRM chunk".to_string()),
        }
    }

    /// Scaled gamma value of a gAMA chunk
    pub fn gama_gamma(&self) -> Result<f64, String> {
        match self {
            PNGChunkData::GAMA { gamma } => {
                Ok(*gamma as f64 / 100000.0)
            },

            _ => Err("PNG: Not a gAMA chunk".to_string()),
        }
    }

    /// Decompress the compressed profile in a iCCP chunk
    pub fn iccp_profile(&self) -> Result<Vec<u8>, String> {
        match self {
            PNGChunkData::ICCP { compression_method, compressed_profile, .. } => {
                match compression_method {
                    PNGCompressionType::Zlib => {
                        Ok(inflate_bytes_zlib(compressed_profile.as_slice())?)
                    }
                }
            },

            _ => Err("PNG: Not a iCCP chunk".to_string()),
        }
    }

    /// Decompress the compressed string in a zTXt chunk
    pub fn ztxt_string(&self) -> Result<String, String> {
        match self {
            PNGChunkData::ZTXT { compression_method, compressed_string, .. } => {
                match compression_method {
                    PNGCompressionType::Zlib => {
                        let bytes = inflate_bytes_zlib(compressed_string.as_slice())?;
                        Ok(String::from_utf8(bytes).unwrap_or(String::new()))
                    }
                }
            },

            _ => Err("PNG: Not a zTXt chunk".to_string()),
        }
    }

    /// Decompress the compressed string in an iTXt chunk
    pub fn itxt_string(&self) -> Result<String, String> {
        match self {
            PNGChunkData::ITXT { compressed, compression_method,
                                 compressed_string, .. } => {
                if *compressed {
                    match compression_method {
                        PNGCompressionType::Zlib => {
                            let bytes = inflate_bytes_zlib(compressed_string.as_slice())?;
                            Ok(String::from_utf8(bytes).unwrap_or(String::new()))
                        }
                    }
                } else {
                    Ok(String::from_utf8(compressed_string.to_vec()).unwrap_or(String::new()))
                }
            },

            _ => Err("PNG: Not an iTXt chunk".to_string()),
        }
    }

    /// Convert the units in a pHYs chunk to pixels per inch
    ///
    /// Yes, it's not SI units. But it's what everyone uses.
    pub fn phys_ppi(&self) -> Result<(f64, f64), String> {
        match self {
            PNGChunkData::PHYS { x_pixels_per_unit, y_pixels_per_unit, unit } => {
                match unit {
                    PNGUnitType::Unknown =>
                        Err("PNG: Unknown unit.".to_string()),

                    PNGUnitType::Metre =>
                        Ok((*x_pixels_per_unit as f64 / 39.370_078_740_157_48,
                            *y_pixels_per_unit as f64 / 39.370_078_740_157_48)),
                }
            },

            _ => Err("PNG: Not a pHYs chunk".to_string()),
        }
    }

    /// Calculate delay from fcTL chunk in seconds
    pub fn fctl_delay(&self) -> Result<f64, String> {
        match self {
            PNGChunkData::FCTL { delay_num, delay_den, .. } => {
                Ok(*delay_num as f64 / *delay_den as f64)
            },

            _ => Err("PNG: Not an fcTL chunk".to_string()),
        }
    }

}


/// Reference to a chunk in a PNG file
#[derive(Copy, Clone, Debug)]
pub struct PNGChunk {
    /// The position in the stream/file for this chunk
    pub position: u64,

    /// Length of this chunk
    pub length: u32,

    /// Chunk type
    pub chunktype: [ u8; 4 ],

    /// Chunk CRC
    pub crc: u32,

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
    for (i, byte) in bytes.iter().enumerate() {
        if *byte == 0 {
            return i;
        }
    }

    bytes.len()
}

impl Default for PNGChunk {
    fn default() -> Self {
        PNGChunk {
            position: 0,
            length: 0,
            chunktype: [ 0_u8; 4 ],
            crc: 0,
        }
    }

}

impl PNGChunk {
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

    /// Read the chunk data and parse it into a PNGChunkData enum
    pub fn read_chunk<R>(&self, stream: &mut R,
                         ihdr: Option<&PNGChunkData>)
                         -> Result<PNGChunkData, std::io::Error>
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
                for _n in 0..num_entries {
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

            "tRNS" => {
                match ihdr.unwrap() {
                    PNGChunkData::IHDR { colour_type, .. } => {
                        match colour_type {
                            PNGColourType::Greyscale => {
                                let mut buf = [ 0_u8; 2 ];
                                chunkstream.read_exact(&mut buf)?;

                                Ok(PNGChunkData::TRNS {
                                    data: PNGtRNSType::Greyscale {
                                        value: u16_be(&buf),
                                    },
                                })
                            },

                            PNGColourType::TrueColour => {
                                let mut buf = [ 0_u8; 6 ];
                                chunkstream.read_exact(&mut buf)?;

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

                                Ok(PNGChunkData::TRNS {
                                    data: PNGtRNSType::IndexedColour {
                                        values,
                                    },
                                })
                            },

                            _ => Err(std::io::Error::other(format!(
                                "PNG: Invalid colour type ({}) in ihdr", *colour_type as u8)))

                        }
                    },

                    _ => Err(std::io::Error::other(
                        "PNG: Wrong chunk type passed as ihdr"))
                }
            },

            "gAMA" => {
                let mut buf = [ 0_u8; 4 ];
                chunkstream.read_exact(&mut buf)?;

                Ok(PNGChunkData::GAMA {
                    gamma: u32_be(&buf),
                })
            },

            "cHRM" => {
                let mut data = Vec::with_capacity(32);
                chunkstream.read_to_end(&mut data)?;

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
                let name_end = find_null(&data);

                Ok(PNGChunkData::ICCP {
                    name: String::from_utf8(data[0..name_end].to_vec()).unwrap_or(String::new()),
                    compression_method: data[name_end].try_into()?,
                    compressed_profile: data[name_end + 2..].to_vec(),
                })
            },

            "sBIT" => {
                match ihdr.unwrap() {
                    PNGChunkData::IHDR { colour_type, .. } => {
                        match colour_type {
                            PNGColourType::Greyscale => {
                                let mut buf = [ 0_u8; 1 ];
                                chunkstream.read_exact(&mut buf)?;

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
                    },

                    _ => Err(std::io::Error::other(
                        "PNG: Wrong chunk type passed as ihdr"))
                }
            },

            "sRGB" => {
                let mut buf = [ 0_u8; 1 ];
                chunkstream.read_exact(&mut buf)?;

                Ok(PNGChunkData::SRGB {
                    rendering_intent: buf[0].try_into()?,
                })
            },

            "cICP" => {
                let mut buf = [ 0_u8; 4 ];
                chunkstream.read_exact(&mut buf)?;

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
                let keyword_end = find_null(&data);

                Ok(PNGChunkData::TEXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .unwrap_or(String::new()),
                    string: String::from_utf8(data[keyword_end + 1..].to_vec())
                        .unwrap_or(String::new()),
                })
            },

            "zTXt" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                let keyword_end = find_null(&data);

                Ok(PNGChunkData::ZTXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .unwrap_or(String::new()),
                    compression_method: data[keyword_end + 1].try_into()?,
                    compressed_string: data[keyword_end + 2..].to_vec(),
                })
            },

            "iTXt" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
                let keyword_end = find_null(&data);
                let language_end = find_null(&data[keyword_end + 3..])
                    + keyword_end + 3;
                let tkeyword_end = find_null(&data[language_end + 1..])
                    + language_end + 1;

                Ok(PNGChunkData::ITXT {
                    keyword: String::from_utf8(data[0..keyword_end].to_vec())
                        .unwrap_or(String::new()),
                    compressed: data[keyword_end + 1] > 0,
                    compression_method: data[keyword_end + 2].try_into()?,
                    language: String::from_utf8(data[keyword_end + 3..language_end]
                                                .to_vec()).unwrap_or(String::new()),
                    translated_keyword: String::from_utf8(data[language_end + 1..tkeyword_end]
                                                          .to_vec()).unwrap_or(String::new()),
                    compressed_string: data[tkeyword_end + 1..].to_vec(),
                })
            },

            "bKGD" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;

                match ihdr.unwrap() {
                    PNGChunkData::IHDR { colour_type, .. } => {
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
                    },

                    _ => Err(std::io::Error::other(
                        "PNG: Wrong chunk type passed as ihdr"))
                }
            },

            "hIST" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
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

                Ok(PNGChunkData::PHYS {
                    x_pixels_per_unit: u32_be(&buf[0..4]),
                    y_pixels_per_unit: u32_be(&buf[4..8]),
                    unit: buf[8].try_into()?,
                })
            },

            "eXIf" => {
                let mut profile = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut profile)?;

                Ok(PNGChunkData::EXIF {
                    profile,
                })
            },

            "sPLT" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;
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
                        .unwrap_or(String::new()),
                    depth,
                    palette,
                })
            },

            "tIME" => {
                let mut buf = [ 0_u8; 7 ];
                chunkstream.read_exact(&mut buf)?;

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

                Ok(PNGChunkData::ACTL {
                    num_frames: u32_be(&buf[0..4]),
                    num_plays: u32_be(&buf[4..8]),
                })
            },

            "fcTL" => {
                let mut buf = [ 0_u8; 26 ];
                chunkstream.read_exact(&mut buf)?;

                Ok(PNGChunkData::FCTL {
                    sequence_number: u32_be(&buf[0..4]),
                    width: u32_be(&buf[4..8]),
                    height: u32_be(&buf[8..12]),
                    x_offset: u32_be(&buf[12..16]),
                    y_offset: u32_be(&buf[16..20]),
                    delay_num: u16_be(&buf[20..22]),
                    delay_den: u16_be(&buf[22..24]),
                    dispose_op: buf[24].try_into()?,
                    blend_op: buf[24].try_into()?,
                })
            },

            "fdAT" => {
                let mut buf = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut buf)?;

                Ok(PNGChunkData::FDAT {
                    sequence_number: u32_be(&buf[0..4]),
                    frame_data: buf[4..].to_vec(),
                })
            },

            "JHDR" => {
                let mut buf = [ 0_u8; 16 ];
                chunkstream.read_exact(&mut buf)?;

                Ok(PNGChunkData::JHDR {
                    width: u32_be(&buf[0..4]),
                    height: u32_be(&buf[4..8]),
                    colour_type: buf[8].try_into()?,
                    image_sample_depth: buf[9].try_into()?,
                    image_compression_method: buf[10].try_into()?,
                    image_interlace_method: buf[11].try_into()?,
                    alpha_sample_depth: buf[12].try_into()?,
                    alpha_compression_method: buf[13].try_into()?,
                    alpha_filter_method: buf[14].try_into()?,
                    alpha_interlace_method: buf[15].try_into()?,
                })
            },

            "JDAT" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;

                Ok(PNGChunkData::JDAT {
                    data,
                })
            },

            "JDAA" => {
                let mut data = Vec::with_capacity(self.length as usize);
                chunkstream.read_to_end(&mut data)?;

                Ok(PNGChunkData::JDAA {
                    data,
                })
            },

            "JSEP" => Ok(PNGChunkData::JSEP),

            _ => Err(std::io::Error::other(format!(
                "PNG: Unhandled chunk type ({})", self.type_str())))
        }
    }

}


/// A reader for reading data from a series of IDAT, JDAT, or JDAA chunks
pub struct DATReader<'a, R>  {
    /// Iterator to the IDAT/JDAT/JDAA chunk(s)
    dat_iter: Iter<'a, PNGChunk>,

    /// The stream that the chunks are read from
    stream: &'a mut R,

    /// A queue of data from the chunks
    buffer: VecDeque<u8>,

}

impl<'a, R> DATReader<'a, R> {
    /// Constructor
    pub fn new(dats: &'a Vec<PNGChunk>, stream: &'a mut R) -> Self {
        DATReader {
            dat_iter: dats.iter(),
            buffer: VecDeque::new(),
            stream,
        }
    }

}

impl<'a, R> Read for DATReader<'a, R>
where R: Read + Seek
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while (self.buffer.len() < buf.len()) && (self.dat_iter.size_hint().0 > 0) {
            let chunkref = self.dat_iter.next().ok_or(std::io::Error::other("Could not get next DAT chunk"))?;
            let chunk = chunkref.read_chunk(self.stream, None)?;
            let data = match chunk {
                PNGChunkData::IDAT { data } => Result::Ok(data),
                PNGChunkData::JDAT { data } => Result::Ok(data),
                PNGChunkData::JDAA { data } => Result::Ok(data),

                _ => Result::Err(std::io::Error::other("chunk is not IDAT, JDAT, or JDAA")),
            }?;
            self.buffer.append(&mut (data.into()));
        }

        let mut len = 0;
        for i in 0..buf.len() {
            let b = self.buffer.pop_front();
            if b.is_none() {
                break;
            }

            buf[i] = b.unwrap();
            len += 1;
        }

        Ok(len)
    }

}
