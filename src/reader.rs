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

use crate::types::*;
use crate::chunks::*;
use crate::crc::*;

#[derive(Copy, Clone, Debug)]
pub enum PNGFileType {
    /// Portable Network Graphics
    PNG,

    /// Multiple-image Network Graphics
    MNG,

    /// JPEG Network Graphics
    JNG,

    /// Animated Portable Network Graphics
    APNG
}

/// A PNG file reader
#[derive(Debug)]
pub struct PNGFileReader<R> {
    /// Image file type
    ///
    /// PNG, MNG, JNG, or APNG
    pub filetype: PNGFileType,

    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Image bit depth per pixel component
    pub bit_depth: u8,

    /// Image colour type
    pub colour_type: PNGColourType,

    /// File stream we're reading from
    pub stream: R,

    /// The list of all chunks in the file
    pub all_chunks: Vec<PNGChunk>,

    /// The IHDR chunk
    pub ihdr: PNGChunkData,

    /// A hashmap of optional chunks that can only appear once in a file,
    /// keyed to their chunk type
    pub optional_chunk_idxs: HashMap<[ u8; 4 ], usize>,

    /// A hashmap of optional chunks that can appear multiple times in a
    /// file, keyed to their chunk type
    pub optional_multi_chunk_idxs: HashMap<[ u8; 4 ], Vec<usize>>,
}

impl<R> PNGFileReader<R>
where R: Read + Seek
{
    /// Constructor from a Read-able type
    fn from_stream(mut stream: R) -> Result<Self, std::io::Error> {
        let mut filetype = PNGFileType::PNG;
        // First check the signature
        {
            let mut signature = [ 0; 8 ];
            stream.read_exact(&mut signature)?;
            match signature {
                [ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ] => {
                    filetype = PNGFileType::PNG;
                },

                [ 0x8a, 0x4d, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ] => {
                    filetype = PNGFileType::MNG;
                },

                [ 0x8b, 0x4a, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ] => {
                    filetype = PNGFileType::JNG;
                },

                _ => {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "PNG: Unrecognised signature"));
                },
            }
        }

        let mut width = 0;
        let mut height = 0;
        let mut bit_depth = 0;
        let mut colour_type = PNGColourType::Greyscale;
        let mut all_chunks = Vec::new();
        let mut ihdr = PNGChunkData::None;
        let mut optional_chunk_idxs = HashMap::new();
        let mut optional_multi_chunk_idxs = HashMap::new();

        // Now just loop reading chunks
        loop {
            let position = stream.stream_position()?;

            let mut buf4 = [ 0_u8; 4 ];
            stream.read_exact(&mut buf4)?;
            let length = u32::from_be_bytes(buf4);

            let mut chunktype = [ 0_u8; 4 ];
            stream.read_exact(&mut chunktype)?;
            let chunktypestr = str::from_utf8(&chunktype).unwrap_or("");

            let mut data_crc = CRC::new();
            data_crc.consume(&chunktype);
            {
                let mut datastream = stream.take(length as u64);
                let mut toread = length;
                let mut buf = [ 0_u8; 65536 ];	// 64 KiB buffer
                while toread > 0 {
                    let readsize = datastream.read(&mut buf)?;
                    data_crc.consume(&buf[0..readsize]);
                    toread -= readsize as u32;
                }

                stream = datastream.into_inner();
            }
            stream.read_exact(&mut buf4)?;
            let crc = u32::from_be_bytes(buf4);
            if crc != data_crc.value() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                                               format!("PNG: Read CRC ({:#x}) doesn't match the computed one ({:#x})",
                                                       crc, data_crc.value())));
            }

            let chunk = PNGChunk {
                position,
                length,
                chunktype,
                crc,
                ancillary: chunktype[0] & 0x20 > 0,
                private: chunktype[1] & 0x20 > 0,
                reserved: chunktype[2] & 0x20 > 0,
                safe_to_copy: chunktype[3] & 0x20 > 0,
            };

            let idx = all_chunks.len();
            all_chunks.push(chunk);

            match chunktypestr {
                "IHDR" => {
                    let oldpos = stream.stream_position()?;
                    // Fill in image metadata
                    ihdr = chunk.read_chunk(&mut stream, None)?;
                    match ihdr {
                        PNGChunkData::IHDR { width, height, bit_depth, colour_type, compression_method: _, filter_method: _, interlace_method: _ } => {
                            width = width;
                            height = height;
                            bit_depth = bit_depth;
                            colour_type = colour_type;
                        },

                        _ => (),
                    }

                    stream.seek(SeekFrom::Start(oldpos))?;
                },

                "IEND" => {
                    break;
                },

                "IDAT" | "fcTL" | "tEXt" | "iTXt" | "zTXt" | "fcTL" | "fdAT" => {
                    if !optional_multi_chunk_idxs.contains_key(&chunktype) {
                        optional_multi_chunk_idxs.insert(chunktype, Vec::new());
                    }
                    optional_multi_chunk_idxs.get_mut(&chunktype).unwrap().push(idx);
                },

                _ => {
                    optional_chunk_idxs.insert(chunktype, idx);
                },
            }

            if (chunktypestr == "aCTL")
                | (chunktypestr == "fcTL")
                | (chunktypestr == "fdAT") {
                    filetype = PNGFileType::APNG;
                }

        }

        Ok(PNGFileReader {
            width,
            height,
            bit_depth,
            colour_type,
            stream,
            all_chunks,
            ihdr,
            optional_chunk_idxs,
            optional_multi_chunk_idxs,
        })
    }

    /*
    pub fn chunks(&self) -> std::slice::Iter<'_, PNGChunk> {
        self.all_chunks.iter()
}
    */

}
