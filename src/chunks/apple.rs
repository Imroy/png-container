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

//! Apple private chunks

use std::io::{Read, Write};

use crate::chunks::PngChunkData;
use crate::crc::CRC;
use crate::to_io_error;

/// Apple's iDOT chunk to allow parallel decoding?
///
/// <https://www.hackerfactor.com/blog/index.php?/archives/895-Connecting-the-iDOTs.html>
#[derive(Clone, Debug, Default)]
pub struct Idot(pub Vec<IdotSegment>);

impl Idot {
    pub(crate) const TYPE: [u8; 4] = *b"iDOT";

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        length: u32,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        if length < 4 {
            return Err(std::io::Error::other(format!(
                "Incorrect length for iDOT chunk: {}",
                length
            )));
        }

        let mut data = vec![0_u8; length as usize];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        // Don't use the number of segments provided, just go off of the chunk length
        let segments = data[4..]
            .chunks(12)
            .map(|bytes| {
                Ok(IdotSegment {
                    start_row: u32::from_be_bytes(bytes[0..4].try_into().map_err(to_io_error)?),
                    num_rows: u32::from_be_bytes(bytes[4..8].try_into().map_err(to_io_error)?),
                    idat_position: u32::from_be_bytes(
                        bytes[8..12].try_into().map_err(to_io_error)?,
                    ),
                })
            })
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        Ok(Self(segments))
    }

    pub(crate) fn length(&self) -> u32 {
        4 + self.0.len() as u32 * 12
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        mut data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        {
            let bytes = (self.0.len() as u32).to_be_bytes();
            stream.write_all(&bytes)?;
            if let Some(ref mut data_crc) = data_crc {
                data_crc.consume(&bytes);
            }
        }

        for segment in &self.0 {
            let sr_bytes = segment.start_row.to_be_bytes();
            stream.write_all(&sr_bytes)?;

            let nr_bytes = segment.num_rows.to_be_bytes();
            stream.write_all(&nr_bytes)?;

            let ip_bytes = segment.idat_position.to_be_bytes();
            stream.write_all(&ip_bytes)?;

            if let Some(ref mut data_crc) = data_crc {
                data_crc.consume(&sr_bytes);
                data_crc.consume(&nr_bytes);
                data_crc.consume(&ip_bytes);
            }
        }

        Ok(())
    }
}

impl From<Idot> for PngChunkData {
    fn from(idot: Idot) -> Self {
        Self::Idot(Box::new(idot))
    }
}

/// A segment in the iDOT chunk
#[derive(Clone, Debug)]
pub struct IdotSegment {
    /// The starting row of this segment
    pub start_row: u32,

    /// The number of rows in the segment
    pub num_rows: u32,

    /// The position of the IDAT where these rows start
    pub idat_position: u32,
}
