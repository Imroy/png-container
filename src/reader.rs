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

/*! PNG/APNG reader
 */

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::str;

use crate::types::*;
use crate::types::*;
use crate::chunks::*;
use crate::streams::*;

/// A frame in an APNG file
#[derive(Clone, Default, Debug)]
pub struct APNGFrame {
    /// The fcTL chunk defining the frame
    pub fctl: PNGChunkRef,

    /// The fdAT chunk(s) containing the frame data
    pub fdats: Vec<PNGChunkRef>,

}


/// A PNG/APNG file reader
#[derive(Debug)]
pub struct PNGSeekableReader<R> {
    /// Image file type
    ///
    /// PNG or APNG
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
    pub all_chunks: Vec<PNGChunkRef>,

    /// The IHDR chunk data
    pub ihdr: PNGChunkData,

    /// The PLTE chunk, if the file has one
    pub plte: Option<PNGChunkRef>,

    /// The IDAT chunk(s)
    pub idats: Vec<PNGChunkRef>,

    /// APNG: List of frames
    pub frames: Vec<APNGFrame>,

    /// The IEND chunk
    pub iend: PNGChunkRef,

    /// A hashmap of optional chunks that can only appear once in a file,
    /// keyed to their chunk type
    pub optional_chunks: HashMap<[ u8; 4 ], PNGChunkRef>,

    /// A hashmap of optional chunks that can appear multiple times in a
    /// file, keyed to their chunk type
    pub optional_multi_chunks: HashMap<[ u8; 4 ], Vec<PNGChunkRef>>,

}

impl<R> PNGSeekableReader<R>
where R: Read + Seek
{
    /// Constructor from a Read-able type
    fn from_stream(mut stream: R) -> Result<Self, std::io::Error> {
        let mut filetype = PNGFileType::PNG;
        // First check the signature
        {
            let mut signature = [ 0; 8 ];
            stream.read_exact(&mut signature)?;
            if signature != [ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ] {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "PNG: Bad signature"));
            }
        }

        let mut width = 0;
        let mut height = 0;
        let mut bit_depth = 0;
        let mut colour_type = PNGColourType::Greyscale;
        let mut all_chunks = Vec::new();
        let mut ihdr = PNGChunkData::None;
        let mut plte = None;
        let mut idats = Vec::new();
        let mut fctl_fdats = Vec::new();
        let mut iend = PNGChunkRef::default();
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

            // Invalid chunk types for PNG files
            if (chunktypestr == "JHDR") | (chunktypestr == "JDAT")
                | (chunktypestr == "JDAA") | (chunktypestr == "JSEP")
            {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                                               format!("PNG: Invalid chunk type \"{}\"",
                                                       chunktypestr)));
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
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                                               format!("PNG: Read CRC ({:#x}) doesn't match the computed one ({:#x})",
                                                       crc, data_crc.value())));
            }

            let chunk = PNGChunkRef {
                position,
                length,
                chunktype,
                crc,
            };

            match chunktypestr {
                "IHDR" => {
                    let oldpos = stream.stream_position()?;
                    // Fill in image metadata
                    ihdr = chunk.read_chunk(&mut stream, None)?;
                    match ihdr {
                        PNGChunkData::IHDR { width, height, colour_type, .. } => {
                            width = width;
                            height = height;
                            bit_depth = bit_depth;
                            colour_type = colour_type;
                        },

                        _ => (),
                    }

                    stream.seek(SeekFrom::Start(oldpos))?;
                },

                "PLTE" => {
                    plte = Some(chunk);
                },

                "IDAT" => {
                    idats.push(chunk);
                },

                "fcTL" | "fdAT" => {
                    fctl_fdats.push(chunk);
                },

                "IEND" => {
                    iend = chunk;
                },

                "tEXt" | "iTXt" | "zTXt" => {
                    optional_multi_chunks.entry(chunktype).or_insert_with(Vec::new);
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

            if (chunktypestr == "aCTL")
                | (chunktypestr == "fcTL")
                | (chunktypestr == "fdAT")
            {
                filetype = PNGFileType::APNG;
            }

        }

        // Sort fcTL and fdAT chunks by their sequence number
        fctl_fdats.sort_by_cached_key(|c| {
            let _ = stream.seek(SeekFrom::Start(c.position + 8));
            c.read_fctl_fdat_sequence_number(&mut stream).unwrap()
        });

        let mut frames = Vec::new();
        let mut frame = APNGFrame::default();

        // Now assemble them into frames
        for chunk in fctl_fdats {
            match chunk.type_str() {
                "fcTL" => {
                    if frame.fctl.position > 0 {
                        frames.push(frame);
                        frame = APNGFrame::default();
                    }
                    frame.fctl = chunk;
                },

                "fdAT" => {
                    frame.fdats.push(chunk);
                },

                _ => (),
            }
        }
        if frame.fctl.position > 0 {
            frames.push(frame);
        }

        Ok(PNGFileReader {
            width,
            height,
            bit_depth,
            colour_type,
            stream,
            all_chunks,
            ihdr,
            plte,
            idats,
            frames,
            iend,
            optional_chunks,
            optional_multi_chunks,
        })
    }

}
