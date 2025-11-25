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

use crate::chunks::*;
use crate::types::*;

/// A PNG/APNG file reader
#[derive(Clone, Debug)]
pub struct PngReader<R> {
    /// Image file type
    ///
    /// PNG or APNG
    pub filetype: PngFileType,

    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Image bit depth per pixel component
    pub bit_depth: u8,

    /// Image colour type
    pub colour_type: PngColourType,

    /// File stream we're reading from
    pub stream: R,

    /// The IHDR chunk data
    pub ihdr: Option<Ihdr>,

    next_chunk_pos: u64,

    in_header: bool,
    first_frame_is_static: bool,
}

impl<R> PngReader<R>
where
    R: Read + Seek,
{
    /// Constructor from a Read-able and Seek-able type
    ///
    /// This just checks the file signature. Use any of the scan_*() methods to read chunks.
    pub fn from_stream(mut stream: R) -> Result<Self, std::io::Error> {
        // First check the signature
        {
            let mut signature = [0; 8];
            stream.read_exact(&mut signature)?;
            if signature != [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a] {
                return Err(std::io::Error::other("PNG: Bad signature"));
            }
        }

        Ok(PngReader {
            filetype: PngFileType::Png,
            width: 0,
            height: 0,
            bit_depth: 0,
            colour_type: PngColourType::Greyscale,
            stream,
            ihdr: None,
            next_chunk_pos: 8,
            in_header: true,
            first_frame_is_static: false,
        })
    }

    /// Scan all of the chunks in a PNG/APNG file
    ///
    /// If this is called after scan_header_chunks(), it will only return the following chunks.
    pub fn scan_all_chunks(&mut self) -> Result<Vec<PngChunkRef>, std::io::Error> {
        let mut chunks = Vec::with_capacity(4);
        loop {
            let chunkref = self.scan_next_chunk()?;
            chunks.push(chunkref);
            if chunkref.chunktype == *b"IEND" {
                break;
            }
        }

        Ok(chunks)
    }

    /// Scan chunks in a PNG/APNG file until the first IDAT chunk
    pub fn scan_header_chunks(&mut self) -> Result<Vec<PngChunkRef>, std::io::Error> {
        let mut chunks = Vec::with_capacity(4);
        loop {
            let chunkref = self.scan_next_chunk()?;
            if chunkref.chunktype == *b"IDAT" {
                self.next_chunk_pos = chunkref.position;
                break;
            }
            chunks.push(chunkref);
        }

        Ok(chunks)
    }

    /// Scan chunks in a PNG/APNG file, returning a Vec of the chunks that match a closure
    pub fn scan_chunks_filtered<F>(&mut self, test: F) -> Result<Vec<PngChunkRef>, std::io::Error>
    where
        F: Fn([u8; 4]) -> bool,
    {
        let mut chunks = Vec::new();
        loop {
            let chunkref = self.scan_next_chunk()?;
            if test(chunkref.chunktype) {
                chunks.push(chunkref);
            }
            if chunkref.chunktype == *b"IEND" {
                break;
            }
        }

        Ok(chunks)
    }

    /// Scan the next chunk
    pub fn scan_next_chunk(&mut self) -> Result<PngChunkRef, std::io::Error> {
        self.stream.seek(SeekFrom::Start(self.next_chunk_pos))?;
        let chunkref = PngChunkRef::from_stream(&mut self.stream)?;

        // Invalid chunk types for PNG/APNG files
        if (chunkref.chunktype == *b"JHDR")
            | (chunkref.chunktype == *b"JDAT")
            | (chunkref.chunktype == *b"JDAA")
            | (chunkref.chunktype == *b"JSEP")
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("PNG: Invalid chunk type \"{:?}\"", chunkref.chunktype),
            ));
        }

        self.next_chunk_pos += 4 + 4 + chunkref.length as u64 + 4;

        match &chunkref.chunktype {
            b"IHDR" => {
                let oldpos = self.stream.stream_position()?;
                // Fill in image metadata
                if let PngChunkData::Ihdr(ihdr) = chunkref.read_chunk(&mut self.stream, None)? {
                    self.ihdr = Some(ihdr);
                    self.width = ihdr.width;
                    self.height = ihdr.height;
                    self.bit_depth = ihdr.bit_depth;
                    self.colour_type = ihdr.colour_type;
                }

                self.stream.seek(SeekFrom::Start(oldpos))?;
            }

            b"IDAT" => {
                self.in_header = false;
            }

            b"aCTL" | b"fdAT" => {
                self.filetype = PngFileType::Apng;
            }

            b"fcTL" => {
                self.filetype = PngFileType::Apng;
                if self.in_header {
                    self.first_frame_is_static = true;
                }
            }

            _ => (),
        }

        Ok(chunkref)
    }

    /// Reset the position of the next chunk to scan back to the start of the file
    pub fn reset_next_chunk_position(&mut self) {
        self.next_chunk_pos = 8;
        self.in_header = true;
    }

    /// Set the position of the next chunk to scan to a given chunk
    pub fn set_next_chunk_position(&mut self, chunkref: &PngChunkRef) {
        self.next_chunk_pos = chunkref.position;
    }

    /// Set the position of the next chunk to scan to after a given chunk
    pub fn set_next_chunk_position_after(&mut self, chunkref: &PngChunkRef) {
        self.next_chunk_pos = chunkref.position + 4 + 4 + chunkref.length as u64 + 4;
    }

    /// Read the chunk data after seeking to the start of its data
    pub fn read_chunk(&mut self, chunkref: &PngChunkRef) -> Result<PngChunkData, std::io::Error> {
        chunkref.read_chunk(&mut self.stream, self.ihdr.as_ref())
    }

    pub fn apng_scan_frames(&mut self) -> std::io::Result<Vec<ApngFrame>> {
        let mut chunkrefs = if self.first_frame_is_static {
            self.scan_chunks_filtered(|ct| ct == *b"IDAT" || ct == *b"fcTL" || ct == *b"fdAT")?
        } else {
            self.scan_chunks_filtered(|ct| ct == *b"fcTL" || ct == *b"fdAT")?
        };

        // Make a Hashmap mapping from IDAT chunk position to a generated sequence number
        let mut idat_seq_nums = HashMap::new();
        if self.first_frame_is_static {
            let mut seq_num = 1;
            chunkrefs
                .iter()
                .filter(|cr| cr.chunktype == *b"IDAT")
                .for_each(|cr| {
                    idat_seq_nums.insert(cr.position, seq_num);
                    seq_num += 1;
                });
        }

        // Sort chunks by their sequence number
        chunkrefs.sort_by_cached_key(|cr| {
            if cr.chunktype == *b"IDAT" {
                idat_seq_nums[&cr.position]
            } else {
                let seq_num = cr.read_fctl_fdat_sequence_number(&mut self.stream).unwrap();
                if self.first_frame_is_static && (seq_num > 0) {
                    seq_num + idat_seq_nums.len() as u32
                } else {
                    seq_num
                }
            }
        });

        // Group fcTL and fdAT chunks into frames
        let mut frames = Vec::new();
        for chunkref in chunkrefs {
            if chunkref.chunktype == *b"fcTL" {
                let chunk = self.read_chunk(&chunkref)?;
                if let PngChunkData::Fctl(fctl) = chunk {
                    frames.push(ApngFrame {
                        fctl: *fctl,
                        dats: Vec::new(),
                    });
                }
            } else {
                if frames.is_empty() {
                    return Err(std::io::Error::other(
                        "At least one fcTL chunk must go before fdAT chunks".to_string(),
                    ));
                }
                let lasti = frames.len() - 1;
                frames[lasti].dats.push(chunkref);
            }
        }

        Ok(frames)
    }
}

/// An APNG frame
#[derive(Clone, Debug)]
pub struct ApngFrame {
    pub fctl: Fctl,

    pub dats: Vec<PngChunkRef>,
}
