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

/*! Generic structs that read from something, process it, and can in turn be read
*/

use std::collections::VecDeque;
use std::io::Read;
use std::slice::Iter;

use crate::chunks::{PNGChunkRef, PNGChunkData};

/// A reader for reading data from a series of IDAT, fdAT, JDAT, or JDAA chunks
pub struct PNGDATReader<'a, R>  {
    /// Iterator to the IDAT/fdAT/JDAT/JDAA chunk(s)
    dat_iter: Iter<'a, PNGChunkRef>,

    /// The stream that the chunks are read from
    stream: &'a mut R,

    /// A queue of data from the chunks
    buffer: VecDeque<u8>,

}

impl<'a, R> PNGDATReader<'a, R> {
    /// Constructor
    pub fn new(dats: &'a Vec<PNGChunkRef>, stream: &'a mut R) -> Self {
        PNGDATReader {
            dat_iter: dats.iter(),
            buffer: VecDeque::new(),
            stream,
        }
    }

}

impl<'a, R> Read for PNGDATReader<'a, R>
where R: Read
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while (self.buffer.len() < buf.len()) && (self.dat_iter.size_hint().0 > 0) {
            let chunkref = self.dat_iter.next().ok_or(std::io::Error::other("Could not get next DAT chunk"))?;
            let chunk = chunkref.read_chunk(self.stream, None)?;
            let data = match chunk {
                PNGChunkData::IDAT { data } => Result::Ok(data),
                PNGChunkData::FDAT { frame_data, .. } => Result::Ok(frame_data),
                PNGChunkData::JDAT { data } => Result::Ok(data),
                PNGChunkData::JDAA { data } => Result::Ok(data),

                _ => Result::Err(std::io::Error::other("chunk is not IDAT, fdAT, JDAT, or JDAA")),
            }?;

            self.buffer.append(&mut (data.into()));
        }

        let mut len = 0;
        for i in 0..buf.len() {
            let b = self.buffer.pop_front();
            if b.is_none() {
                break;
            }

            buf[i] = b.unwrap();
            len += 1;
        }

        Ok(len)
    }

}
