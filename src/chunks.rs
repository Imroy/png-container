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
use uom::si::{
    f64::{LinearNumberDensity, Luminance, Time},
    linear_number_density::per_meter,
    luminance::candela_per_square_meter,

pub mod animation;
pub mod colour_space;
pub mod critical;
pub mod extensions;
pub mod misc;
pub mod text;

pub use crate::chunks::{
    animation::*, colour_space::*, critical::*, extensions::*, misc::*, text::*,
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
    Ihdr(Box<Ihdr>),

    /// Palette
    Plte(Box<Plte>),

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
        ihdr: Option<&Ihdr>,
    ) -> Result<PngChunkData, std::io::Error>
    where
        R: Read + Seek,
    {
        stream.seek(SeekFrom::Start(self.position + 4 + 4))?;
        let mut chunkstream = stream.take(self.length as u64);

        let mut data_crc = CRC::new();
        data_crc.consume(&self.chunktype);

        let chunk = match &self.chunktype {
            b"IHDR" => Ok(PngChunkData::Ihdr(Box::new(Ihdr::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            b"PLTE" => Ok(PngChunkData::Plte(Box::new(Plte::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            b"IDAT" => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Idat(Box::new(data)))
            }

            b"IEND" => Ok(PngChunkData::Iend),

            b"tRNS" => {
                if let Some(Ihdr { colour_type, .. }) = ihdr {
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
                    Err(std::io::Error::other("PNG: Unset ihdr".to_string()))
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

            b"cHRM" => Ok(PngChunkData::Chrm(Box::new(Chrm::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            b"iCCP" => Ok(PngChunkData::Iccp(Box::new(Iccp::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            b"sBIT" => {
                if let Some(Ihdr { colour_type, .. }) = ihdr {
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
                    Err(std::io::Error::other("PNG: Unset ihdr".to_string()))
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

            b"mDCV" => Ok(PngChunkData::Mdcv(Box::new(Mdcv::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            b"cLLI" => Ok(PngChunkData::Clli(Box::new(Clli::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            b"tEXt" => Ok(PngChunkData::Text(Box::new(Text::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            b"zTXt" => Ok(PngChunkData::Ztxt(Box::new(Ztxt::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            b"iTXt" => Ok(PngChunkData::Itxt(Box::new(Itxt::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            b"bKGD" => {
                if let Some(Ihdr { colour_type, .. }) = ihdr {
                    let mut data = vec![0_u8; self.length as usize];
                    chunkstream.read_exact(&mut data)?;
                    data_crc.consume(&data);

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
                    Err(std::io::Error::other("PNG: Unset ihdr".to_string()))
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

            b"sPLT" => Ok(PngChunkData::Splt(Box::new(Splt::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

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

            b"fcTL" => Ok(PngChunkData::Fctl(Box::new(Fctl::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            b"fdAT" => Ok(PngChunkData::Fdat(Box::new(Fdat::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

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

            b"pCAL" => Ok(PngChunkData::Pcal(Box::new(Pcal::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            b"sCAL" => Ok(PngChunkData::Scal(Box::new(Scal::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

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

            b"gIFx" => Ok(PngChunkData::Gifx(Box::new(Gifx::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

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
