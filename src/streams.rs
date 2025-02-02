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

use crate::chunks::PNGDATChunkIter;

/// A reader for reading data from a series of IDAT, fdAT, JDAT, or JDAA chunks
pub struct PNGDATReader<'a, R> {
    /// Iterator to the IDAT/fdAT/JDAT/JDAA chunk(s)
    dat_iter: PNGDATChunkIter<'a, R>,

    /// A queue of data from the chunks
    queue: VecDeque<u8>,
}

impl<'a, R> PNGDATReader<'a, R> {
    /// Constructor
    ///
    /// `dat_iter`: an iterator over IDAT, fdAT, JDAT, or JDAA chunks.
    pub fn new(dat_iter: PNGDATChunkIter<'a, R>) -> Self {
        Self {
            dat_iter,
            queue: VecDeque::new(),
        }
    }

    fn get_next_chunk(&mut self) -> bool
    where
        R: Read,
    {
        let chunk = self.dat_iter.next();
        if let Some(chunk) = chunk {
            let data_iter = chunk.dat_data_iter();
            if let Some(data_iter) = data_iter {
                self.queue.extend(data_iter);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl<'a, R> Read for PNGDATReader<'a, R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while self.queue.len() < buf.len() {
            if !self.get_next_chunk() {
                break;
            }
        }

        let len = if self.queue.len() >= buf.len() {
            buf.len()
        } else {
            self.queue.len()
        };

        self.queue
            .drain(0..len)
            .enumerate()
            .for_each(|(i, b)| buf[i] = b);

        Ok(len)
    }
}
