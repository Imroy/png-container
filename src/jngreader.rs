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

/*! JNG reader
 */

use std::io::{Read, Seek, SeekFrom};

use crate::chunks::*;
use crate::types::*;

/// A JNG file reader
#[derive(Debug)]
pub struct JNGSeekableReader<R> {
    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Image colour type
    pub colour_type: JNGColourType,

    /// File stream we're reading from
    pub stream: R,

    /// The JHDR chunk data
    pub jhdr: PNGChunkData,

    /// The IEND chunk
    pub iend: PNGChunkRef,

    next_chunk_pos: u64,
}

impl<R> JNGSeekableReader<R>
where
    R: Read + Seek,
{
    /// Constructor from a Read-able type
    fn from_stream(mut stream: R) -> Result<Self, std::io::Error> {
        // First check the signature
        {
            let mut signature = [0; 8];
            stream.read_exact(&mut signature)?;
            if signature != [0x8b, 0x4a, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a] {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "JNG: Bad file signature",
                ));
            }
        }

        Ok(JNGSeekableReader {
            width: 0,
            height: 0,
            colour_type: JNGColourType::Greyscale,
            stream,
            jhdr: PNGChunkData::None,
            iend: PNGChunkRef::default(),
            next_chunk_pos: 8,
        })
    }

    /// Scan all of the chunks in a JNG file
    pub fn scan_all_chunks(&mut self) -> Result<Vec<PNGChunkRef>, std::io::Error> {
        let mut chunks = Vec::new();
        loop {
            let chunkref = self.scan_next_chunk()?;
            chunks.push(chunkref);
            if chunkref.chunktype == *b"IEND" {
                break;
            }
        }

        Ok(chunks)
    }

    /// Scan chunks in a JNG file until the first IDAT or JDAT chunk
    pub fn scan_header_chunks(&mut self) -> Result<Vec<PNGChunkRef>, std::io::Error> {
        let mut chunks = Vec::new();
        loop {
            let chunkref = self.scan_next_chunk()?;
            if chunkref.chunktype == *b"IDAT" || chunkref.chunktype == *b"JDAT" {
                self.next_chunk_pos = chunkref.position;
                break;
            }
            chunks.push(chunkref);
        }

        Ok(chunks)
    }

    /// Scan the next chunk
    pub fn scan_next_chunk(&mut self) -> Result<PNGChunkRef, std::io::Error> {
        self.stream.seek(SeekFrom::Start(self.next_chunk_pos))?;
        let chunkref = PNGChunkRef::from_stream(&mut self.stream)?;

        // Invalid chunk types for JNG files
        if (chunkref.chunktype == *b"PLTE")
            | (chunkref.chunktype == *b"hIST")
            | (chunkref.chunktype == *b"pCAL")
            | (chunkref.chunktype == *b"sBIT")
            | (chunkref.chunktype == *b"sPLT")
            | (chunkref.chunktype == *b"tRNS")
            | (chunkref.chunktype == *b"fRAc")
            | (chunkref.chunktype == *b"gIFg")
            | (chunkref.chunktype == *b"gIFx")
            | (chunkref.chunktype == *b"aCTL")
            | (chunkref.chunktype == *b"fcTL")
            | (chunkref.chunktype == *b"fdAT")
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("JNG: Invalid chunk type \"{:?}\"", chunkref.chunktype),
            ));
        }

        self.next_chunk_pos += 4 + 4 + chunkref.length as u64 + 4;

        match &chunkref.chunktype {
            b"JHDR" => {
                let oldpos = self.stream.stream_position()?;
                // Fill in image metadata
                self.jhdr = chunkref.read_chunk(&mut self.stream, None)?;
                if let PNGChunkData::JHDR {
                    width,
                    height,
                    colour_type,
                    ..
                } = self.jhdr
                {
                    self.width = width;
                    self.height = height;
                    self.colour_type = colour_type;
                }

                self.stream.seek(SeekFrom::Start(oldpos))?;
            }

            b"IEND" => {
                self.iend = chunkref;
            }

            _ => (),
        }

        Ok(chunkref)
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
    where
        R: Read + Seek,
    {
        self.stream.seek(SeekFrom::Start(chunkref.position + 8))?;
        chunkref.read_chunk(&mut self.stream, None)
    }
}
