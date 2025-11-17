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

use chrono::{DateTime, Utc};
use uom::si::f64::{LinearNumberDensity, Time as UoMTime};

pub mod animation;
pub mod colour_space;
pub mod critical;
pub mod extensions;
pub mod misc;
pub mod text;
pub mod time;
pub mod transparency;

pub use crate::chunks::{
    animation::*, colour_space::*, critical::*, extensions::*, misc::*, text::*, time::*,
    transparency::*,
};

use crate::crc::*;

/// Enum of PNG chunk types and the data they hold
#[derive(Clone, Debug)]
pub enum PngChunkData {
    /// Empty type
    None,

    // Critical chunks
    /// Image header
    Ihdr(Ihdr),

    /// Palette
    Plte(Box<Plte>),

    /// Image data
    Idat(Box<Vec<u8>>),

    /// Image end
    Iend,

    // Transparency information
    /// Transparency
    Trns(Box<Trns>),

    // Colour space information
    /// Primary chromaticities and white point
    Chrm(Box<Chrm>),

    /// Image gamma
    Gama(Gama),

    /// Embedded ICC profile
    Iccp(Box<Iccp>),

    /// Significant bits
    Sbit(Sbit),

    /// Standard RGB colour space
    Srgb(Srgb),

    /// Coding-independent code points for video signal type identification
    Cicp(Cicp),

    /// Mastering Display Color Volume
    Mdcv(Box<Mdcv>),

    /// Content Light Level Information
    Clli(Clli),

    // Textual information
    /// Textual data
    Text(Box<Text>),

    /// Compressed textual data
    Ztxt(Box<Ztxt>),

    /// International textual data
    Itxt(Box<Itxt>),

    // Miscellaneous information
    /// Background colour
    Bkgd(Bkgd),

    /// Image histogram
    Hist(Box<Hist>),

    /// Physical pixel dimensions
    Phys(Phys),

    /// Suggested palette
    Splt(Box<Splt>),

    /// Exchangeable Image File (Exif) Profile
    Exif(Box<Vec<u8>>),

    // Time stamp information
    /// Image last-modification time
    Time(Time),

    /// Animation control
    Actl(Actl),

    /// Frame control
    Fctl(Box<Fctl>),

    /// Frame data
    Fdat(Box<Fdat>),

    // Extensions
    /// Image offset
    Offs(Offs),

    /// Calibration of pixel values
    Pcal(Box<Pcal>),

    /// Physical scale of image subject
    Scal(Box<Scal>),

    /// GIF Graphic Control Extension
    Gifg(Gifg),

    /// GIF Application Extension
    Gifx(Box<Gifx>),

    /// Indicator of Stereo Image
    Ster(Ster),

    // JNG chunks
    /// JNG header
    Jhdr(Box<Jhdr>),

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
        if let PngChunkData::Gama(g) = self {
            Some(g.gamma())
        } else {
            None
        }
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
        if let PngChunkData::Phys(phys) = self {
            return phys.resolution();
        }

        None
    }

    /// Convert the timestamp in a tIME chunk to a chrono DateTime object
    pub fn time(&self) -> Option<DateTime<Utc>> {
        if let PngChunkData::Time(time) = self {
            return time.time();
        }

        None
    }

    /// Calculate delay from fcTL chunk in seconds
    pub fn fctl_delay(&self) -> Option<UoMTime> {
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
    pub fn from_stream<R>(stream: &mut R) -> Result<Self, std::io::Error>
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
    pub fn read_fctl_fdat_sequence_number<R>(&self, stream: &mut R) -> Result<u32, std::io::Error>
    where
        R: Read + Seek,
    {
        match &self.chunktype {
            b"fcTL" | b"fdAT" => {
                stream.seek(SeekFrom::Start(self.position + 4 + 4))?;
                let mut buf4 = [0_u8; 4];
                stream.read_exact(&mut buf4)?;
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
    /// `ihdr`: The IHDR chunk, only used for tRNS, sBIT, and bKGD chunks for the colour_type value.
    /// This also checks the chunk CRC value.
    pub fn read_chunk<R>(
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

        let chunk = match self.chunktype {
            Ihdr::TYPE => Ok(PngChunkData::Ihdr(Ihdr::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            Plte::TYPE => Ok(PngChunkData::Plte(Box::new(Plte::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            IDAT_TYPE => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Idat(Box::new(data)))
            }

            IEND_TYPE => Ok(PngChunkData::Iend),

            Trns::TYPE => {
                if let Some(Ihdr { colour_type, .. }) = ihdr {
                    Ok(PngChunkData::Trns(Box::new(Trns::from_stream(
                        &mut chunkstream,
                        self.length,
                        *colour_type,
                        Some(&mut data_crc),
                    )?)))
                } else {
                    Err(std::io::Error::other("PNG: Unset ihdr".to_string()))
                }
            }

            Gama::TYPE => Ok(PngChunkData::Gama(Gama::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            Chrm::TYPE => Ok(PngChunkData::Chrm(Box::new(Chrm::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            Iccp::TYPE => Ok(PngChunkData::Iccp(Box::new(Iccp::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Sbit::TYPE => {
                if let Some(Ihdr { colour_type, .. }) = ihdr {
                    Ok(PngChunkData::Sbit(Sbit::from_stream(
                        &mut chunkstream,
                        self.length,
                        *colour_type,
                        Some(&mut data_crc),
                    )?))
                } else {
                    Err(std::io::Error::other("PNG: Unset ihdr".to_string()))
                }
            }

            Srgb::TYPE => Ok(PngChunkData::Srgb(Srgb::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            Cicp::TYPE => Ok(PngChunkData::Cicp(Cicp::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            Mdcv::TYPE => Ok(PngChunkData::Mdcv(Box::new(Mdcv::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            Clli::TYPE => Ok(PngChunkData::Clli(Clli::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            Text::TYPE => Ok(PngChunkData::Text(Box::new(Text::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Ztxt::TYPE => Ok(PngChunkData::Ztxt(Box::new(Ztxt::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Itxt::TYPE => Ok(PngChunkData::Itxt(Box::new(Itxt::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Bkgd::TYPE => {
                if let Some(Ihdr { colour_type, .. }) = ihdr {
                    Ok(PngChunkData::Bkgd(Bkgd::from_stream(
                        &mut chunkstream,
                        self.length,
                        *colour_type,
                        Some(&mut data_crc),
                    )?))
                } else {
                    Err(std::io::Error::other("PNG: Unset ihdr".to_string()))
                }
            }

            Hist::TYPE => Ok(PngChunkData::Hist(Box::new(Hist::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Phys::TYPE => Ok(PngChunkData::Phys(Phys::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            EXIF_TYPE => {
                let mut profile = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut profile)?;
                data_crc.consume(&profile);

                Ok(PngChunkData::Exif(Box::new(profile)))
            }

            Splt::TYPE => Ok(PngChunkData::Splt(Box::new(Splt::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Time::TYPE => Ok(PngChunkData::Time(Time::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            // Animation information
            Actl::TYPE => Ok(PngChunkData::Actl(Actl::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            Fctl::TYPE => Ok(PngChunkData::Fctl(Box::new(Fctl::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            Fdat::TYPE => Ok(PngChunkData::Fdat(Box::new(Fdat::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            // Extensions
            Offs::TYPE => Ok(PngChunkData::Offs(Offs::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            Pcal::TYPE => Ok(PngChunkData::Pcal(Box::new(Pcal::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Scal::TYPE => Ok(PngChunkData::Scal(Box::new(Scal::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Gifg::TYPE => Ok(PngChunkData::Gifg(Gifg::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            Gifx::TYPE => Ok(PngChunkData::Gifx(Box::new(Gifx::from_stream(
                &mut chunkstream,
                self.length,
                Some(&mut data_crc),
            )?))),

            Ster::TYPE => Ok(PngChunkData::Ster(Ster::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?)),

            // JNG chunks
            Jhdr::TYPE => Ok(PngChunkData::Jhdr(Box::new(Jhdr::from_stream(
                &mut chunkstream,
                Some(&mut data_crc),
            )?))),

            JDAT_TYPE => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Jdat(Box::new(data)))
            }

            JDAA_TYPE => {
                let mut data = vec![0_u8; self.length as usize];
                chunkstream.read_exact(&mut data)?;
                data_crc.consume(&data);

                Ok(PngChunkData::Jdaa(Box::new(data)))
            }

            JSEP_TYPE => Ok(PngChunkData::Jsep),

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

pub(crate) const IDAT_TYPE: [u8; 4] = *b"IDAT";
pub(crate) const IEND_TYPE: [u8; 4] = *b"IEND";
pub(crate) const EXIF_TYPE: [u8; 4] = *b"eXIf";
pub(crate) const JDAT_TYPE: [u8; 4] = *b"JDAT";
pub(crate) const JDAA_TYPE: [u8; 4] = *b"JDAA";
pub(crate) const JSEP_TYPE: [u8; 4] = *b"JSEP";

/// A frame in an APNG file
#[derive(Clone, Default, Debug)]
pub struct ApngFrame {
    /// The fcTL chunk defining the frame
    pub fctl: PngChunkRef,

    /// The fdAT chunk(s) containing the frame data
    pub fdats: Vec<PngChunkRef>,
}
