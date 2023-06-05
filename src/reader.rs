/*
  png-container
  Copyright (C) 2023 Ian Tester

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

/*! PNG reader
 */

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::str;

use crate::chunks::*;

/// A PNG file reader
#[derive(Debug)]
pub struct PNGFileReader<R> {
    pub width: u32,

    pub height: u32,

    pub bit_depth: u8,

    pub colour_type: PNGColourType,

    pub stream: R,

    pub all_chunks: Vec<PNGChunk>,

    ihdr_idx: usize,
    iend_idx: usize,

    optional_chunk_idxs: HashMap<[ u8; 4 ], usize>,
    optional_multi_chunk_idxs: HashMap<[ u8; 4 ], Vec<usize>>,
}

impl<R> PNGFileReader<R>
where R: Read + Seek
{
    /// Constructor from a Read-able type
    fn from_stream(mut stream: R) -> Result<Self, std::io::Error> {
        // First check the signature
        {
            let mut buf = [ 0; 8 ];
            stream.read_exact(&mut buf)?;
            if buf != [ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ] {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "PNG: Bad signature"));
            }
        }

        let mut fr = PNGFileReader {
            width: 0,
            height: 0,
            bit_depth: 0,
            colour_type: PNGColourType::Greyscale,
            stream,
            all_chunks: Vec::new(),
            ihdr_idx: 0,
            iend_idx: 0,
            optional_chunk_idxs: HashMap::new(),
            optional_multi_chunk_idxs: HashMap::new(),
        };

        // Now just loop reading chunks
        loop {
            let position = fr.stream.stream_position()?;

            let mut buf4 = [ 0_u8; 4 ];
            fr.stream.read_exact(&mut buf4)?;
            let length = u32::from_be_bytes(buf4);

            let mut chunktype = [ 0_u8; 4 ];
            fr.stream.read_exact(&mut chunktype)?;
            let chunktypestr = str::from_utf8(&chunktype).unwrap();

            fr.stream.seek(SeekFrom::Current(length as i64))?;
            // TODO: check CRC
            fr.stream.read_exact(&mut buf4)?;
            let crc = u32::from_be_bytes(buf4);

            let idx = fr.all_chunks.len();

            fr.all_chunks.push(PNGChunk {
                position,
                length,
                chunktype,
                crc,
                ancillary: chunktype[0] & 0x20 > 0,
                private: chunktype[1] & 0x20 > 0,
                reserved: chunktype[2] & 0x20 > 0,
                safe_to_copy: chunktype[3] & 0x20 > 0,
            });

            match chunktypestr {
                "IHDR" => {
                    fr.ihdr_idx = idx;
                    let oldpos = fr.stream.stream_position()?;
                    // Fill in image metadata
                    let ihdr = fr.all_chunks[idx].read_chunk(&mut fr.stream)?;
                    match ihdr {
                        PNGChunkData::IHDR { width, height, bit_depth, colour_type, compression_method: _, filter_method: _, interlace_method: _ } => {
                            fr.width = width;
                            fr.height = height;
                            fr.bit_depth = bit_depth;
                            fr.colour_type = colour_type;
                        },

                        _ => (),
                    }

                    fr.stream.seek(SeekFrom::Start(oldpos))?;
                },

                "IEND" => {
                    fr.iend_idx = idx;
                    break;
                },

                "IDAT" | "fcTL" | "tEXt" | "iTXt" | "zTXt" => {
                    if !fr.optional_multi_chunk_idxs.contains_key(&chunktype) {
                        fr.optional_multi_chunk_idxs.insert(chunktype, Vec::new());
                    }
                    fr.optional_multi_chunk_idxs.get_mut(&chunktype).unwrap().push(idx);
                },

                _ => {
                    fr.optional_chunk_idxs.insert(chunktype, idx);
                },
            }
        }

        Ok(fr)
    }

    /*
    pub fn chunks(&self) -> std::slice::Iter<'_, PNGChunk> {
        self.all_chunks.iter()
}
    */

    pub fn ihdr(&self) -> PNGChunk {
        self.all_chunks[self.ihdr_idx]
    }

}
