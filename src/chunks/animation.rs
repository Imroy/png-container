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

use std::io::{Read, Write};

use uom::si::f64::Time;

use crate::chunks::PngChunkData;
use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Animation control
#[derive(Clone, Copy, Debug)]
pub struct Actl {
    pub num_frames: u32,
    pub num_plays: u32,
}

impl Actl {
    pub(crate) const TYPE: [u8; 4] = *b"acTL";
    pub(crate) const LENGTH: u32 = 8;

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 8];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            num_frames: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            num_plays: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
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
        let frames_bytes = self.num_frames.to_be_bytes();
        stream.write_all(&frames_bytes)?;

        let plays_bytes = self.num_plays.to_be_bytes();
        stream.write_all(&plays_bytes)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&frames_bytes);
            data_crc.consume(&plays_bytes);
        }

        Ok(())
    }
}

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
    pub(crate) const TYPE: [u8; 4] = *b"fcTL";
    pub(crate) const LENGTH: u32 = 26;

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
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

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let seqnum_bytes = self.sequence_number.to_be_bytes();
        stream.write_all(&seqnum_bytes)?;

        let width_bytes = self.width.to_be_bytes();
        stream.write_all(&width_bytes)?;

        let height_bytes = self.height.to_be_bytes();
        stream.write_all(&height_bytes)?;

        let xoff_bytes = self.x_offset.to_be_bytes();
        stream.write_all(&xoff_bytes)?;

        let yoff_bytes = self.y_offset.to_be_bytes();
        stream.write_all(&yoff_bytes)?;

        let delayn_bytes = self.delay_num.to_be_bytes();
        stream.write_all(&delayn_bytes)?;

        let delayd_bytes = self.delay_den.to_be_bytes();
        stream.write_all(&delayd_bytes)?;

        let end_bytes = [self.dispose_op.into(), self.blend_op.into()];
        stream.write_all(&end_bytes)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&seqnum_bytes);
            data_crc.consume(&width_bytes);
            data_crc.consume(&height_bytes);
            data_crc.consume(&xoff_bytes);
            data_crc.consume(&yoff_bytes);
            data_crc.consume(&delayn_bytes);
            data_crc.consume(&delayd_bytes);
            data_crc.consume(&end_bytes);
        }

        Ok(())
    }

    /// Calculate delay from fcTL chunk in seconds
    pub fn delay(&self) -> Time {
        Time::new::<uom::si::time::second>(self.delay_num as f64 / self.delay_den as f64)
    }
}

impl PngChunkData {
    /// Calculate delay from fcTL chunk in seconds
    pub fn fctl_delay(&self) -> Option<Time> {
        if let PngChunkData::Fctl(fctl) = self {
            return Some(fctl.delay());
        }

        None
    }
}

/// Frame data
#[derive(Clone, Debug)]
pub struct Fdat {
    pub sequence_number: u32,
    pub frame_data: Vec<u8>,
}

impl Fdat {
    pub(crate) const TYPE: [u8; 4] = *b"fdAT";

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
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

    pub(crate) fn length(&self) -> u32 {
        4 + self.frame_data.len() as u32
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let seq_bytes = self.sequence_number.to_be_bytes();
        stream.write_all(&seq_bytes)?;

        stream.write_all(&self.frame_data)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&seq_bytes);
            data_crc.consume(&self.frame_data);
        }

        Ok(())
    }
}
