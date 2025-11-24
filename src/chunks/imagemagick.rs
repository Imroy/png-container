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

//! Some chunk types used by ImageMagick

use std::io::{Read, Write};

use num_enum::{FromPrimitive, IntoPrimitive};

use crate::chunks::PngChunkData;
use crate::crc::*;
use crate::to_io_error;

/// Canvas
#[derive(Clone, Copy, Debug)]
pub struct Canv {
    pub width: u32,
    pub height: u32,
    pub x_offset: i32,
    pub y_offset: i32,
}

impl Canv {
    pub(crate) const TYPE: [u8; 4] = *b"caNv";
    pub(crate) const LENGTH: u32 = 16;

    /// Constructor
    pub fn new(width: u32, height: u32, x_offset: i32, y_offset: i32) -> Self {
        Self {
            width,
            height,
            x_offset,
            y_offset,
        }
    }

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 16];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            width: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            height: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            x_offset: i32::from_be_bytes(data[8..12].try_into().map_err(to_io_error)?),
            y_offset: i32::from_be_bytes(data[12..16].try_into().map_err(to_io_error)?),
        })
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let width_bytes = self.width.to_be_bytes();
        stream.write_all(&width_bytes)?;

        let height_bytes = self.height.to_be_bytes();
        stream.write_all(&height_bytes)?;

        let x_off_bytes = self.x_offset.to_be_bytes();
        stream.write_all(&x_off_bytes)?;

        let y_off_bytes = self.y_offset.to_be_bytes();
        stream.write_all(&y_off_bytes)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&width_bytes);
            data_crc.consume(&height_bytes);
            data_crc.consume(&x_off_bytes);
            data_crc.consume(&y_off_bytes);
        }

        Ok(())
    }
}

impl From<Canv> for PngChunkData {
    fn from(canv: Canv) -> Self {
        Self::Canv(Box::new(canv))
    }
}

/// VirtualPage
#[derive(Clone, Copy, Debug)]
pub struct Vpag {
    pub virtual_page_width: u32,
    pub virtual_page_height: u32,

    // Units?
    pub virtual_page_units: u8,
}

impl Vpag {
    pub(crate) const TYPE: [u8; 4] = *b"vpAg";
    pub(crate) const LENGTH: u32 = 9;

    /// Constructor
    pub fn new(virtual_page_width: u32, virtual_page_height: u32, virtual_page_units: u8) -> Self {
        Self {
            virtual_page_width,
            virtual_page_height,
            virtual_page_units,
        }
    }

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 9];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            virtual_page_width: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            virtual_page_height: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            virtual_page_units: data[8],
        })
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let width_bytes = self.virtual_page_width.to_be_bytes();
        stream.write_all(&width_bytes)?;

        let height_bytes = self.virtual_page_height.to_be_bytes();
        stream.write_all(&height_bytes)?;

        stream.write_all(&[self.virtual_page_units])?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&width_bytes);
            data_crc.consume(&height_bytes);
            data_crc.consume(&[self.virtual_page_units]);
        }

        Ok(())
    }
}

impl From<Vpag> for PngChunkData {
    fn from(vpag: Vpag) -> Self {
        Self::Vpag(Box::new(vpag))
    }
}

/// Orientation (from EXIF/TIFF tag 0x0112)
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum PngOrientation {
    Undefined,
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
    LeftTop,
    RightTop,
    RightBottom,
    LeftBottom,

    /// The catch-all variant used by [num_enum::FromPrimitive]
    #[num_enum(catch_all)]
    Other(u8),
}
