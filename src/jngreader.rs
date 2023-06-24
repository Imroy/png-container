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
use crate::crc::*;
use crate::types::JNGColourType;

/// A JNG file reader
#[derive(Debug)]
pub struct JNGFileReader<R> {
    /// File stream we're reading from
    pub stream: R,

    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Image colour type
    pub colour_type: JNGColourType,

    /// The list of all chunks in the file
    pub all_chunks: Vec<PNGChunk>,

    /// The JHDR chunk data
    pub jhdr: PNGChunkData,

    /// The IDAT chunks
    pub idats: Vec<PNGChunk>,

    /// The JDAT chunks
    pub jdats: Vec<PNGChunk>,

    /// A second list of JDAT chunks for the 12-bit image when
    /// image_sample_depth == Depth8And12
    pub jdats2: Vec<PNGChunk>,

    /// The JDAA chunks
    pub jdaas: Vec<PNGChunk>,

    /// The IEND chunk
    pub iend: PNGChunk,

    /// A hashmap of optional chunks that can only appear once in a file,
    /// keyed to their chunk type
    pub optional_chunks: HashMap<[ u8; 4 ], PNGChunk>,

    /// A hashmap of optional chunks that can appear multiple times in a
    /// file, keyed to their chunk type
    pub optional_multi_chunks: HashMap<[ u8; 4 ], Vec<PNGChunk>>,
}

impl<R> JNGFileReader<R>
where R: Read + Seek
{
    /// Constructor from a Read-able type
    fn from_stream(mut stream: R) -> Result<Self, std::io::Error> {
        // First check the signature
        {
            let mut signature = [ 0; 8 ];
            stream.read_exact(&mut signature)?;
            if signature != [ 0x8b, 0x4a, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ] {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "JNG: Bad file signature"));
            }
        }

        let mut width = 0;
        let mut height = 0;
        let mut colour_type = JNGColourType::Greyscale;
        let mut all_chunks = Vec::new();
        let mut jhdr = PNGChunkData::None;
        let mut idats = Vec::new();
        let mut jdats = Vec::new();
        let mut jdats2 = Vec::new();
        let mut first_image = true;
        let mut jdaas = Vec::new();
        let mut iend = PNGChunk::default();
        let mut optional_chunks = HashMap::new();
        let mut optional_multi_chunks = HashMap::new();

        // Now just loop reading chunks
        loop {
            let position = stream.stream_position()?;

            let mut buf4 = [ 0_u8; 4 ];
            stream.read_exact(&mut buf4)?;
            let length = u32::from_be_bytes(buf4);

            let mut chunktype = [ 0_u8; 4 ];
            stream.read_exact(&mut chunktype)?;
            let chunktypestr = str::from_utf8(&chunktype).unwrap_or("");

            // Invalid chunk types for JNG files
            if (chunktypestr == "PLTE") | (chunktypestr == "hIST")
                | (chunktypestr == "pCAL") | (chunktypestr == "sBIT")
                | (chunktypestr == "sPLT") | (chunktypestr == "tRNS")
                | (chunktypestr == "fRAc") | (chunktypestr == "gIFg")
                | (chunktypestr == "gIFx") | (chunktypestr == "aCTL")
                | (chunktypestr == "fcTL") | (chunktypestr == "fdAT")
            {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JNG: Invalid chunk type \"{}\"", chunktypestr)));
            }

            let mut data_crc = CRC::new();
            data_crc.consume(&chunktype);
            {
                let mut datastream = stream.take(length as u64);
                let mut toread = length;
                let mut buf = [ 0_u8; 65536 ];	// 64 KiB buffer
                while toread > 0 {
                    let readsize = datastream.read(&mut buf).unwrap_or(0);
                    data_crc.consume(&buf[0..readsize]);
                    toread -= readsize as u32;
                }

                stream = datastream.into_inner();
            }
            stream.read_exact(&mut buf4)?;
            let crc = u32::from_be_bytes(buf4);
            if crc != data_crc.value() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JNG: Read CRC ({:#x}) doesn't match the computed one ({:#x})", crc, data_crc.value())));
            }

            let chunk = PNGChunk {
                position,
                length,
                chunktype,
                crc,
            };

            match chunktypestr {
                "JHDR" => {
                    let oldpos = stream.stream_position()?;
                    // Fill in image metadata
                    jhdr = chunk.read_chunk(&mut stream, None)?;
                    match jhdr {
                        PNGChunkData::JHDR { width, height, colour_type, .. } => {
                            width = width;
                            height = height;
                            colour_type = colour_type;
                        },
                    }

                    stream.seek(SeekFrom::Start(oldpos))?;
                },

                "IDAT" => {
                    idats.push(chunk);
                },

                "JDAT" => {
                    if first_image {
                        jdats.push(chunk);
                    } else {
                        jdats2.push(chunk);
                    }
                },

                "JDAA" => {
                    jdaas.push(chunk);
                },

                "IEND" => {
                    iend = chunk;
                },

                "tEXt" | "iTXt" | "zTXt" => {
                    optional_multi_chunks.entry(chunktype).or_insert_with(|| Vec::new());
                    optional_multi_chunks.get_mut(&chunktype).unwrap().push(chunk);
                },

                _ => {
                    optional_chunks.insert(chunktype, chunk);
                },
            }

            all_chunks.push(chunk);

            if chunktypestr == "IEND" {
                break;
            }

            if chunktypestr == "JSEP" {
                first_image = false;
            }

        }

        Ok(JNGFileReader {
            width,
            height,
            colour_type,
            stream,
            all_chunks,
            jhdr,
            idats,
            jdats,
            jdats2,
            jdaas,
            iend,
            optional_chunks,
            optional_multi_chunks,
        })
    }

}
