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

//! Critical chunks (IHDR, PLTE)

use std::io::Read;

use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Image header
#[derive(Clone, Copy, Debug)]
pub struct Ihdr {
    /// Width of image in pixels
    pub width: u32,

    /// Height of image in pixels
    pub height: u32,

    /// Number of bits per sample
    pub bit_depth: u8,

    /// Colour type
    pub colour_type: PngColourType,

    /// Compression method
    pub compression_method: PngCompressionMethod,

    /// Filter method
    pub filter_method: PngFilterMethod,

    /// Interlace method
    pub interlace_method: PngInterlaceMethod,
}

impl Ihdr {
    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 13];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            width: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            height: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            bit_depth: data[8],
            colour_type: data[9].try_into().map_err(to_io_error)?,
            compression_method: data[10].try_into().map_err(to_io_error)?,
            filter_method: data[11].try_into().map_err(to_io_error)?,
            interlace_method: data[12].try_into().map_err(to_io_error)?,
        })
    }
}

/// Palette
#[derive(Clone, Debug, Default)]
pub struct Plte(pub Vec<PngPaletteEntry>);

impl Plte {
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

        Ok(Self(
            data.chunks(3)
                .map(|col| {
                    Ok(PngPaletteEntry {
                        red: col[0],
                        green: col[1],
                        blue: col[2],
                    })
                })
                .collect::<Result<Vec<_>, std::io::Error>>()?,
        ))
    }
}
