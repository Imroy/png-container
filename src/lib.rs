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

/*! The Portable Network Graphics format
 *
 * Handles [PNG](https://en.wikipedia.org/wiki/PNG), [APNG](https://en.wikipedia.org/wiki/APNG),
 * and [JNG](https://en.wikipedia.org/wiki/JPEG_Network_Graphics) files.
 * Maybe [MNG](https://en.wikipedia.org/wiki/Multiple-image_Network_Graphics) in the future.
 */

pub mod chunks;
pub mod crc;
pub mod jngreader;
pub mod reader;
pub mod types;

pub fn to_io_error<T>(e: T) -> std::io::Error
where
    T: ToString,
{
    std::io::Error::other(e.to_string())
}
