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

//! Transparency chunk

use std::io::Read;

use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// tRNS chunk
#[derive(Clone, Debug)]
pub enum Trns {
    Greyscale { value: u16 },

    TrueColour { red: u16, green: u16, blue: u16 },

    IndexedColour { values: Vec<u8> },
}

impl Trns {
    pub(crate) const TYPE: [u8; 4] = *b"tRNS";

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
            PngColourType::Greyscale => Ok(Self::Greyscale {
                value: u16::from_be_bytes(data[0..2].try_into().map_err(to_io_error)?),
            }),

            PngColourType::TrueColour => Ok(Self::TrueColour {
                red: u16::from_be_bytes(data[0..2].try_into().map_err(to_io_error)?),
                green: u16::from_be_bytes(data[2..4].try_into().map_err(to_io_error)?),
                blue: u16::from_be_bytes(data[4..6].try_into().map_err(to_io_error)?),
            }),

            PngColourType::IndexedColour => Ok(Self::IndexedColour { values: data }),

            _ => Err(std::io::Error::other(format!(
                "PNG: Invalid colour type ({}) in ihdr",
                colour_type as u8
            ))),
        }
    }
}
