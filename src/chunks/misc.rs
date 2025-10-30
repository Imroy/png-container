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

use std::io::Read;

use uom::si::{f64::LinearNumberDensity, linear_number_density::per_meter};

use crate::chunks::find_null;
use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Suggested palette
#[derive(Clone, Debug, Default)]
pub struct Splt {
    pub name: String,
    pub depth: u8,
    pub palette: Vec<PngSuggestedPaletteEntry>,
}

impl Splt {
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
}
