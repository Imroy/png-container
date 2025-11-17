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

//! Public extension chunks

use std::io::Read;

use crate::chunks::find_null;
use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Image offset
#[derive(Clone, Copy, Debug)]
pub struct Offs {
    pub x: u32,
    pub y: u32,
    pub unit: PngUnitType,
}

impl Offs {
    pub(crate) const TYPE: [u8; 4] = *b"oFFs";

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 9];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            x: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            y: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            unit: data[8].try_into().map_err(to_io_error)?,
        })
    }
}

/// Calibration of pixel values
#[derive(Clone, Debug)]
pub struct Pcal {
    pub name: String,
    pub original_zero: u32,
    pub original_max: u32,
    pub equation_type: CalibrationEquationType,
    pub unit_name: String,
    pub parameters: Vec<String>,
}

impl Pcal {
    pub(crate) const TYPE: [u8; 4] = *b"pCAL";

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

        let name_end = find_null(&data);
        let num_parameters = data[name_end + 9];
        let unit_end = find_null(&data[name_end + 10..]) + name_end + 10;

        let parameters = data[unit_end..]
            .split(|b| *b == 0)
            .map(|slice| slice.iter().map(|b| *b as char).collect::<String>())
            .collect::<Vec<_>>();
        if parameters.len() != num_parameters as usize {
            return Err(std::io::Error::other(format!(
                "Read {} parameters but there are supposed to be {}",
                parameters.len(),
                num_parameters
            )));
        }

        Ok(Self {
            name: data[0..name_end].iter().map(|b| *b as char).collect(),
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
            equation_type: data[name_end + 8].try_into().map_err(to_io_error)?,
            unit_name: data[name_end + 10..unit_end]
                .iter()
                .map(|b| *b as char)
                .collect(),
            parameters,
        })
    }
}

/// Physical scale of image subject
#[derive(Clone, Debug)]
pub struct Scal {
    pub unit: PngUnitType,
    pub pixel_width: String,
    pub pixel_height: String,
}

impl Scal {
    pub(crate) const TYPE: [u8; 4] = *b"sCAL";

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

        let width_end = find_null(&data[1..]) + 1;
        let height_end = find_null(&data[width_end..]) + width_end;

        Ok(Self {
            unit: data[0].try_into().map_err(to_io_error)?,
            pixel_width: data[1..width_end].iter().map(|b| *b as char).collect(),
            pixel_height: data[width_end..height_end]
                .iter()
                .map(|b| *b as char)
                .collect(),
        })
    }
}

/// GIF Graphic Control Extension
#[derive(Clone, Copy, Debug)]
pub struct Gifg {
    pub disposal_method: GifDisposalMethod,
    pub user_input: bool,
    pub delay_time: u16,
}

impl Gifg {
    pub(crate) const TYPE: [u8; 4] = *b"gIFg";

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 4];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            disposal_method: data[0].into(),
            user_input: data[1] > 0,
            delay_time: u16::from_be_bytes(data[2..].try_into().map_err(to_io_error)?),
        })
    }
}

/// GIF Application Extension
#[derive(Clone, Debug)]
pub struct Gifx {
    pub app_id: String,
    pub app_auth: [u8; 3],
    pub app_data: Vec<u8>,
}

impl Gifx {
    pub(crate) const TYPE: [u8; 4] = *b"gIFx";

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
            app_id: data[0..8].iter().map(|b| *b as char).collect(),
            app_auth: [data[8], data[9], data[10]],
            app_data: data[11..].to_vec(),
        })
    }
}

/// Indicator of Stereo Image
#[derive(Clone, Copy, Debug)]
pub struct Ster {
    pub mode: StereoMode,
}

impl Ster {
    pub(crate) const TYPE: [u8; 4] = *b"sTER";

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 1];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            mode: data[0].try_into().map_err(to_io_error)?,
        })
    }
}
