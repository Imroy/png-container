/*
  png-container
  Copyright (C) 2025 Ian Tester

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

//! tIME chunk

use std::io::{Read, Write};

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

use crate::crc::*;
use crate::to_io_error;

/// Image last-modification time
#[derive(Copy, Clone, Debug)]
pub struct Time {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl Time {
    pub(crate) const TYPE: [u8; 4] = *b"tIME";
    pub(crate) const LENGTH: u32 = 7;

    /// Read contents from a stream
    pub fn from_stream<R>(stream: &mut R, data_crc: Option<&mut CRC>) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 7];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            year: u16::from_be_bytes(data[0..2].try_into().map_err(to_io_error)?),
            month: data[2],
            day: data[3],
            hour: data[4],
            minute: data[5],
            second: data[6],
        })
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let year_bytes = self.year.to_be_bytes();
        let data = [
            year_bytes[0],
            year_bytes[1],
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        ];
        stream.write_all(&data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(())
    }

    pub fn time(&self) -> Option<DateTime<Utc>> {
        Some(DateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(self.year as i32, self.month as u32, self.day as u32)?,
                NaiveTime::from_hms_opt(self.hour as u32, self.minute as u32, self.second as u32)?,
            ),
            Utc,
        ))
    }
}
