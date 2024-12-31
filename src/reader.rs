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

use std::io::{Read, Seek, SeekFrom};

use crate::types::*;
use crate::chunks::*;

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

    /// The IHDR chunk data
    pub ihdr: PNGChunkData,

    /// The PLTE chunk, if the file has one
    pub plte: Option<PNGChunkRef>,

    /// The IEND chunk
    pub iend: PNGChunkRef,

    next_chunk_pos: u64,

}

impl<R> PNGSeekableReader<R>
where R: Read + Seek
{
    /// Constructor from a Read-able and Seek-able type
    ///
    /// This just checks the file signature. Use any of the scan_*() methods to read chunks.
    fn from_stream(mut stream: R) -> Result<Self, std::io::Error> {
        // First check the signature
        {
            let mut signature = [ 0; 8 ];
            stream.read_exact(&mut signature)?;
            if signature != [ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ] {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "PNG: Bad signature"));
            }
        }

        Ok(PNGSeekableReader {
            filetype: PNGFileType::PNG,
            width: 0,
            height: 0,
            bit_depth: 0,
            colour_type: PNGColourType::Greyscale,
            stream,
            ihdr: PNGChunkData::None,
            plte: None,
            iend: PNGChunkRef::default(),
            next_chunk_pos: 8,
        })
    }

    /// Scan all of the chunks in a PNG/APNG file
    ///
    /// If this is called after scan_header_chunks(), it will only return the following chunks.
    pub fn scan_all_chunks(&mut self) -> Result<Vec<PNGChunkRef>, std::io::Error> {
        let mut chunks = Vec::with_capacity(4);
        loop {
            let chunk = self.scan_next_chunk()?;
            chunks.push(chunk);
            if chunk.chunktype == *b"IEND" {
                break;
            }
        }

        Ok(chunks)
    }

    /// Scan chunks in a PNG/APNG file until the first IDAT chunk
    pub fn scan_header_chunks(&mut self) -> Result<Vec<PNGChunkRef>, std::io::Error> {
        let mut chunks = Vec::with_capacity(4);
        loop {
            let chunk = self.scan_next_chunk()?;
            if chunk.chunktype == *b"IDAT" {
                self.next_chunk_pos = chunk.position;
                break;
            }
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    /// Scan chunks in a PNG/APNG file, returning a Vec of the chunks that match a closure
    pub fn scan_chunks_filtered<F>(&mut self, test: F) -> Result<Vec<PNGChunkRef>, std::io::Error>
    where F: Fn([ u8; 4 ]) -> bool
    {
        let mut chunks = Vec::new();
        loop {
            let chunk = self.scan_next_chunk()?;
            if test(chunk.chunktype) {
                chunks.push(chunk);
            }
            if chunk.chunktype == *b"IEND" {
                break;
            }
        }

        Ok(chunks)
    }

    /// Scan the next chunk
    pub fn scan_next_chunk(&mut self) -> Result<PNGChunkRef, std::io::Error> {
        self.stream.seek(SeekFrom::Start(self.next_chunk_pos))?;
        let position = self.stream.stream_position()?;
        let chunk = PNGChunkRef::new(&mut self.stream, position)?;

        // Invalid chunk types for PNG/APNG files
        if (chunk.chunktype == *b"JHDR") | (chunk.chunktype == *b"JDAT")
            | (chunk.chunktype == *b"JDAA") | (chunk.chunktype == *b"JSEP")
        {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                                           format!("PNG: Invalid chunk type \"{:?}\"",
                                                   chunk.chunktype)));
        }

        self.next_chunk_pos += 4 + 4 + chunk.length as u64 + 4;

        match &chunk.chunktype {
            b"IHDR" => {
                let oldpos = self.stream.stream_position()?;
                // Fill in image metadata
                self.ihdr = chunk.read_chunk(&mut self.stream, None)?;
                match self.ihdr {
                    PNGChunkData::IHDR { width, height, bit_depth, colour_type, .. } => {
                        self.width = width;
                        self.height = height;
                        self.bit_depth = bit_depth;
                        self.colour_type = colour_type;
                    },

                     _ => (),
                }

                self.stream.seek(SeekFrom::Start(oldpos))?;
            },

            b"PLTE" => {
                self.plte = Some(chunk);
            },

            b"IEND" => {
                self.iend = chunk;
            },

            _ => (),
        }

        if (chunk.chunktype == *b"aCTL")
            | (chunk.chunktype == *b"fcTL")
            | (chunk.chunktype == *b"fdAT")
        {
            self.filetype = PNGFileType::APNG;
        }

        Ok(chunk)
    }

    /// Reset the position of the next chunk to scan back to the start of the file
    pub fn reset_next_chunk_position(&mut self) {
        self.next_chunk_pos = 8;
    }

    /// Set the position of the next chunk to scan to a given chunk
    pub fn set_next_chunk_position(&mut self, chunkref: &PNGChunkRef) {
        self.next_chunk_pos = chunkref.position;
    }

    /// Set the position of the next chunk to scan to after a given chunk
    pub fn set_next_chunk_position_after(&mut self, chunkref: &PNGChunkRef) {
        self.next_chunk_pos = chunkref.position + 4 + 4 + chunkref.length as u64 + 4;
    }

    /// Read the chunk data after seeking to the start of its data
    pub fn read_chunk(&mut self, chunkref: &PNGChunkRef) -> Result<PNGChunkData, std::io::Error>
    where R: Read + Seek
    {
        self.stream.seek(SeekFrom::Start(chunkref.position + 8))?;
        chunkref.read_chunk(&mut self.stream, Some(&self.ihdr))
    }

    pub fn apng_scan_frames(&mut self) -> Result<Vec<APNGFrame>, std::io::Error> {
        let mut fctl_fdats = self.scan_chunks_filtered(|ct| ct == *b"fcTL" || ct == *b"fdAT")?;

        // Sort fcTL and fdAT chunks by their sequence number
        fctl_fdats.sort_by_cached_key(|c| {
            let _ = self.stream.seek(SeekFrom::Start(c.position + 8));
            c.read_fctl_fdat_sequence_number(&mut self.stream).unwrap()
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

        Ok(frames)
    }

}
