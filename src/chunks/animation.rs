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

//! Animation chunks

use std::io::Read;

use uom::si::f64::Time;

use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Frame control
#[derive(Clone, Copy, Debug)]
pub struct Fctl {
    pub sequence_number: u32,
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub delay_num: u16,
    pub delay_den: u16,
    pub dispose_op: ApngDisposalOperator,
    pub blend_op: ApngBlendOperator,
}

impl Fctl {
    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 26];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            sequence_number: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            width: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            height: u32::from_be_bytes(data[8..12].try_into().map_err(to_io_error)?),
            x_offset: u32::from_be_bytes(data[12..16].try_into().map_err(to_io_error)?),
            y_offset: u32::from_be_bytes(data[16..20].try_into().map_err(to_io_error)?),
            delay_num: u16::from_be_bytes(data[20..22].try_into().map_err(to_io_error)?),
            delay_den: u16::from_be_bytes(data[22..24].try_into().map_err(to_io_error)?),
            dispose_op: data[24].try_into().map_err(to_io_error)?,
            blend_op: data[24].try_into().map_err(to_io_error)?,
        })
    }

    /// Calculate delay from fcTL chunk in seconds
    pub fn delay(&self) -> Time {
        Time::new::<uom::si::time::second>(self.delay_num as f64 / self.delay_den as f64)
    }
}

/// Frame data
#[derive(Clone, Debug)]
pub struct Fdat {
    pub sequence_number: u32,
    pub frame_data: Vec<u8>,
}

impl Fdat {
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

        Ok(Self {
            sequence_number: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            frame_data: data[4..].to_vec(),
        })
    }
}
