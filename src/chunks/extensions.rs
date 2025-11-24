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

use std::io::{Read, Write};

use uom::si::{f64::Length, length::meter};

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
    pub(crate) const LENGTH: u32 = 9;

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
            x: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            y: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            unit: data[8].try_into().map_err(to_io_error)?,
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
        let x_bytes = self.x.to_be_bytes();
        stream.write_all(&x_bytes)?;

        let y_bytes = self.y.to_be_bytes();
        stream.write_all(&y_bytes)?;

        let unit_byte = [self.unit.into()];
        stream.write_all(&unit_byte)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&x_bytes);
            data_crc.consume(&y_bytes);
            data_crc.consume(&unit_byte);
        }

        Ok(())
    }

    /// Convert the units to a UoM type
    pub fn offset(&self) -> Option<(Length, Length)> {
        match self.unit {
            PngUnitType::Unknown => None,

            PngUnitType::Metre => Some((
                Length::new::<meter>(self.x as f64),
                Length::new::<meter>(self.y as f64),
            )),
        }
    }
}

impl PngChunkData {
    /// Convert the units in an oFFs chunk to a UoM type
    pub fn offs_offset(&self) -> Option<(Length, Length)> {
        if let Self::Offs(offs) = self {
            return offs.offset();
        }

        None
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

    pub(crate) fn length(&self) -> u32 {
        self.name.len() as u32
            + 1
            + 9
            + 1
            + self.unit_name.len() as u32
            + self.parameters.iter().fold(1, |a, p| a + p.len() as u32)
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let name_bytes = self.name.chars().map(|c| c as u8).collect::<Vec<u8>>();
        stream.write_all(&name_bytes)?;

        let null = [0];
        stream.write_all(&null)?;

        let zero_bytes = self.original_zero.to_be_bytes();
        stream.write_all(&zero_bytes)?;

        let max_bytes = self.original_max.to_be_bytes();
        stream.write_all(&max_bytes)?;

        let mid_bytes = [self.equation_type.into(), self.parameters.len() as u8];
        stream.write_all(&mid_bytes)?;

        let unit_bytes = self.unit_name.chars().map(|c| c as u8).collect::<Vec<u8>>();
        stream.write_all(&unit_bytes)?;

        for param in &self.parameters {
            stream.write_all(&null)?;
            let param_bytes = param.chars().map(|c| c as u8).collect::<Vec<u8>>();
            stream.write_all(&param_bytes)?;
        }

        if let Some(data_crc) = data_crc {
            data_crc.consume(&name_bytes);
            data_crc.consume(&null);
            data_crc.consume(&zero_bytes);
            data_crc.consume(&max_bytes);
            data_crc.consume(&mid_bytes);
            data_crc.consume(&unit_bytes);
            for param in &self.parameters {
                let param_bytes = param.chars().map(|c| c as u8).collect::<Vec<u8>>();
                data_crc.consume(&null);
                data_crc.consume(&param_bytes);
            }
        }

        Ok(())
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

    pub(crate) fn length(&self) -> u32 {
        1 + self.pixel_width.len() as u32 + 1 + self.pixel_height.len() as u32
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let unit_byte = [self.unit.into()];
        stream.write_all(&unit_byte)?;

        let width_bytes = self
            .pixel_width
            .chars()
            .map(|c| c as u8)
            .collect::<Vec<u8>>();
        stream.write_all(&width_bytes)?;

        let null = [0];
        stream.write_all(&null)?;

        let height_bytes = self
            .pixel_height
            .chars()
            .map(|c| c as u8)
            .collect::<Vec<u8>>();
        stream.write_all(&height_bytes)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&unit_byte);
            data_crc.consume(&width_bytes);
            data_crc.consume(&null);
            data_crc.consume(&height_bytes);
        }

        Ok(())
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
    pub(crate) const LENGTH: u32 = 4;

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
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

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let start_bytes = [self.disposal_method.into(), self.user_input.into()];
        stream.write_all(&start_bytes)?;

        let delay_bytes = self.delay_time.to_be_bytes();
        stream.write_all(&delay_bytes)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&start_bytes);
            data_crc.consume(&delay_bytes);
        }

        Ok(())
    }
}

/// GIF Application Extension
#[derive(Clone, Debug)]
pub struct Gifx {
    pub app_id: [char; 8],
    pub app_auth: [u8; 3],
    pub app_data: Vec<u8>,
}

impl Gifx {
    pub(crate) const TYPE: [u8; 4] = *b"gIFx";

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
            app_id: data[0..8]
                .iter()
                .map(|b| *b as char)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|e| std::io::Error::other(format!("Couldn't convert {:?}", e)))?,
            app_auth: [data[8], data[9], data[10]],
            app_data: data[11..].to_vec(),
        })
    }

    pub(crate) fn length(&self) -> u32 {
        8 + 3 + self.app_data.len() as u32
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let id_bytes = self.app_id.iter().map(|c| *c as u8).collect::<Vec<u8>>();
        stream.write_all(&id_bytes)?;

        stream.write_all(&self.app_auth)?;
        stream.write_all(&self.app_data)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&id_bytes);
            data_crc.consume(&self.app_auth);
            data_crc.consume(&self.app_data);
        }

        Ok(())
    }
}

/// Indicator of Stereo Image
#[derive(Clone, Copy, Debug)]
pub struct Ster {
    pub mode: StereoMode,
}

impl Ster {
    pub(crate) const TYPE: [u8; 4] = *b"sTER";
    pub(crate) const LENGTH: u32 = 1;

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
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

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let mode_byte = [self.mode.into()];
        stream.write_all(&mode_byte)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&mode_byte);
        }

        Ok(())
    }
}
