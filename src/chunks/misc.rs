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

//! Miscellaneous chunks

use std::io::{Read, Write};

use uom::si::{f64::LinearNumberDensity, linear_number_density::per_meter};

use crate::chunks::{PngChunkData, find_null};
use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Background colour
#[derive(Clone, Copy, Debug)]
pub enum Bkgd {
    Greyscale { value: u16 },

    TrueColour { red: u16, green: u16, blue: u16 },

    IndexedColour { index: u8 },
}

impl Bkgd {
    pub(crate) const TYPE: [u8; 4] = *b"bKGD";

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
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
            PngColourType::Greyscale | PngColourType::GreyscaleAlpha => {
                if length != 2 {
                    return Err(std::io::Error::other(format!(
                        "PNG: Invalid length of bKGD chunk ({})",
                        length
                    )));
                }

                Ok(Self::Greyscale {
                    value: u16::from_be_bytes(data[0..2].try_into().map_err(to_io_error)?),
                })
            }

            PngColourType::TrueColour | PngColourType::TrueColourAlpha => {
                if length != 6 {
                    return Err(std::io::Error::other(format!(
                        "Png: Invalid length of bKGD chunk ({})",
                        length
                    )));
                }

                Ok(Self::TrueColour {
                    red: u16::from_be_bytes(data[0..2].try_into().map_err(to_io_error)?),
                    green: u16::from_be_bytes(data[2..4].try_into().map_err(to_io_error)?),
                    blue: u16::from_be_bytes(data[4..6].try_into().map_err(to_io_error)?),
                })
            }

            PngColourType::IndexedColour => {
                if length != 1 {
                    return Err(std::io::Error::other(format!(
                        "Png: Invalid length of bKGD chunk ({})",
                        length
                    )));
                }

                Ok(Self::IndexedColour { index: data[0] })
            }
        }
    }

    pub(crate) fn length(&self) -> u32 {
        match self {
            Bkgd::Greyscale { .. } => 2,
            Bkgd::TrueColour { .. } => 6,
            Bkgd::IndexedColour { .. } => 1,
        }
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let data: &[u8] = match self {
            Bkgd::Greyscale { value } => &value.to_be_bytes(),
            Bkgd::TrueColour { red, green, blue } => {
                let r = red.to_be_bytes();
                let g = green.to_be_bytes();
                let b = blue.to_be_bytes();
                &[r[0], r[1], g[0], g[1], b[0], b[1]]
            }
            Bkgd::IndexedColour { index } => &[*index],
        };
        stream.write_all(data)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(data);
        }

        Ok(())
    }
}

impl From<Bkgd> for PngChunkData {
    fn from(bkgd: Bkgd) -> Self {
        Self::Bkgd(bkgd)
    }
}

/// Image histogram
#[derive(Clone, Debug)]
pub struct Hist(pub Vec<u16>);

impl Hist {
    pub(crate) const TYPE: [u8; 4] = *b"hIST";

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

        Ok(Self(
            data.chunks(2)
                .map(|h| Ok(u16::from_be_bytes(h.try_into().map_err(to_io_error)?)))
                .collect::<Result<Vec<_>, std::io::Error>>()?,
        ))
    }

    pub(crate) fn length(&self) -> u32 {
        self.0.len() as u32 * 2
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let data = self
            .0
            .iter()
            .flat_map(|h| h.to_be_bytes())
            .collect::<Vec<u8>>();
        stream.write_all(&data)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(())
    }
}

impl From<Hist> for PngChunkData {
    fn from(hist: Hist) -> Self {
        Self::Hist(Box::new(hist))
    }
}

/// Physical pixel dimensions
#[derive(Clone, Copy, Debug)]
pub struct Phys {
    pub x_pixels_per_unit: u32,
    pub y_pixels_per_unit: u32,
    pub unit: PngUnitType,
}

impl Phys {
    pub(crate) const TYPE: [u8; 4] = *b"pHYs";
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
            x_pixels_per_unit: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            y_pixels_per_unit: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
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
        let x_bytes = self.x_pixels_per_unit.to_be_bytes();
        stream.write_all(&x_bytes)?;

        let y_bytes = self.y_pixels_per_unit.to_be_bytes();
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
    pub fn resolution(&self) -> Option<(LinearNumberDensity, LinearNumberDensity)> {
        match self.unit {
            PngUnitType::Unknown => None,

            PngUnitType::Metre => Some((
                LinearNumberDensity::new::<per_meter>(self.x_pixels_per_unit as f64),
                LinearNumberDensity::new::<per_meter>(self.y_pixels_per_unit as f64),
            )),
        }
    }
}

impl PngChunkData {
    /// Convert the units in a pHYs chunk to a UoM type
    pub fn phys_res(&self) -> Option<(LinearNumberDensity, LinearNumberDensity)> {
        if let Self::Phys(phys) = self {
            return phys.resolution();
        }

        None
    }
}

impl From<Phys> for PngChunkData {
    fn from(phys: Phys) -> Self {
        Self::Phys(phys)
    }
}

/// Suggested palette
#[derive(Clone, Debug, Default)]
pub struct Splt {
    pub name: String,
    pub depth: u8,
    pub palette: Vec<PngSuggestedPaletteEntry>,
}

impl Splt {
    pub(crate) const TYPE: [u8; 4] = *b"sPLT";

    /// Constructor
    pub fn new(name: &str, depth: u8, palette: &[PngSuggestedPaletteEntry]) -> Self {
        Self {
            name: name.into(),
            depth,
            palette: palette.into(),
        }
    }

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
        let depth = data[name_end + 1];
        let entry_size = ((depth / 8) * 4) + 2;
        let num_entries = (length as usize - name_end - 1) / (entry_size as usize);

        Ok(Self {
            name: data[0..name_end].iter().map(|b| *b as char).collect(),
            depth,
            palette: (0..num_entries)
                .map(|i| {
                    let start = name_end + 2 + (i * entry_size as usize);
                    if depth == 8 {
                        Ok(PngSuggestedPaletteEntry {
                            red: data[start] as u16,
                            green: data[start + 1] as u16,
                            blue: data[start + 2] as u16,
                            alpha: data[start + 3] as u16,
                            frequency: u16::from_be_bytes(
                                data[start + 4..start + 6].try_into().map_err(to_io_error)?,
                            ),
                        })
                    } else {
                        Ok(PngSuggestedPaletteEntry {
                            red: u16::from_be_bytes(
                                data[start..start + 2].try_into().map_err(to_io_error)?,
                            ),
                            green: u16::from_be_bytes(
                                data[start + 2..start + 4].try_into().map_err(to_io_error)?,
                            ),
                            blue: u16::from_be_bytes(
                                data[start + 4..start + 6].try_into().map_err(to_io_error)?,
                            ),
                            alpha: u16::from_be_bytes(
                                data[start + 6..start + 8].try_into().map_err(to_io_error)?,
                            ),
                            frequency: u16::from_be_bytes(
                                data[start + 8..start + 10]
                                    .try_into()
                                    .map_err(to_io_error)?,
                            ),
                        })
                    }
                })
                .collect::<Result<Vec<_>, std::io::Error>>()?,
        })
    }

    pub(crate) fn length(&self) -> u32 {
        let entry_size = ((self.depth as u32 / 8) * 4) + 2;
        self.name.len() as u32 + 1 + 1 + (self.palette.len() as u32 * entry_size)
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

        let mid_bytes = [0, self.depth];
        stream.write_all(&mid_bytes)?;

        let pal = if self.depth == 8 {
            self.palette
                .iter()
                .flat_map(|p| {
                    let freq_bytes = p.frequency.to_be_bytes();
                    [
                        (p.red & 0xff) as u8,
                        (p.green & 0xff) as u8,
                        (p.blue & 0xff) as u8,
                        freq_bytes[0],
                        freq_bytes[1],
                    ]
                })
                .collect()
        } else {
            self.palette
                .iter()
                .flat_map(|p| {
                    let red_bytes = p.red.to_be_bytes();
                    let green_bytes = p.green.to_be_bytes();
                    let blue_bytes = p.blue.to_be_bytes();
                    let freq_bytes = p.frequency.to_be_bytes();
                    [
                        red_bytes[0],
                        red_bytes[1],
                        green_bytes[0],
                        green_bytes[1],
                        blue_bytes[0],
                        blue_bytes[1],
                        freq_bytes[0],
                        freq_bytes[1],
                    ]
                })
                .collect::<Vec<u8>>()
        };
        stream.write_all(&pal)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&name_bytes);
            data_crc.consume(&mid_bytes);
            data_crc.consume(&pal);
        }

        Ok(())
    }
}

impl From<Splt> for PngChunkData {
    fn from(splt: Splt) -> Self {
        Self::Splt(Box::new(splt))
    }
}

/// Exchangeable Image File (Exif) Profile
#[derive(Clone, Debug)]
pub struct Exif(pub Vec<u8>);

impl Exif {
    pub(crate) const TYPE: [u8; 4] = *b"eXIf";

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

        Ok(Self(data))
    }

    pub(crate) fn length(&self) -> u32 {
        self.0.len() as u32
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        stream.write_all(&self.0)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&self.0);
        }

        Ok(())
    }

}

impl From<Exif> for PngChunkData {
    fn from(exif: Exif) -> Self {
        Self::Exif(Box::new(exif))
    }
}
