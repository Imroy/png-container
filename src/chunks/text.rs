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

//! tEXt, zTXt, and iTXt chunks

use std::io::Read;

use flate2::{
    Compression,
    bufread::{ZlibDecoder, ZlibEncoder},
};

use crate::chunks::find_null;
use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Textual data
#[derive(Clone, Debug)]
pub struct Text {
    pub keyword: String,
    pub string: String,
}

impl Text {
    /// Read contents from a stream
    pub fn from_stream<R>(
        stream: &mut R,
        length: u32,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = vec![0_u8; length as usize];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        let keyword_end = find_null(&data);
        Ok(Self {
            keyword: data[0..keyword_end].iter().map(|b| *b as char).collect(),
            string: data[keyword_end + 1..].iter().map(|b| *b as char).collect(),
        })
    }
}

/// Compressed textual data
#[derive(Clone, Debug, Default)]
pub struct Ztxt {
    pub keyword: String,
    pub compression_method: PngCompressionMethod,
    pub compressed_string: Vec<u8>,
}

impl Ztxt {
    /// Constructor
    pub fn new(keyword: &str, compression_method: PngCompressionMethod, string: &str) -> Self {
        let mut compressed_string = Vec::new();
        if compression_method == PngCompressionMethod::Zlib {
            let mut encoder = ZlibEncoder::new(string.as_bytes(), Compression::best());
            let _ = encoder.read_to_end(&mut compressed_string);
        }

        Self {
            keyword: keyword.to_string(),
            compression_method,
            compressed_string,
        }
    }

    /// Read contents from a stream
    pub fn from_stream<R>(
        stream: &mut R,
        length: u32,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = vec![0_u8; length as usize];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        let keyword_end = find_null(&data);
        Ok(Self {
            keyword: data[0..keyword_end].iter().map(|b| *b as char).collect(),
            compression_method: data[keyword_end + 1].try_into().map_err(to_io_error)?,
            compressed_string: data[keyword_end + 2..].to_vec(),
        })
    }

    /// Set the string
    pub fn set_string(&mut self, compression_method: PngCompressionMethod, string: &str) {
        let mut compressed_string = Vec::new();
        if compression_method == PngCompressionMethod::Zlib {
            let mut encoder = ZlibEncoder::new(string.as_bytes(), Compression::best());
            let _ = encoder.read_to_end(&mut compressed_string);
        }

        self.compression_method = compression_method;
        self.compressed_string = compressed_string;
    }

    /// Decompress the compressed string in a zTXt chunk
    pub fn string(&self) -> Option<String> {
        if self.compression_method == PngCompressionMethod::Zlib {
            let mut decoder = ZlibDecoder::new(self.compressed_string.as_slice());
            let mut out = Vec::new();
            if decoder.read_to_end(&mut out).is_ok() {
                return Some(out.iter().map(|b| *b as char).collect());
            }
        }

        None
    }
}

/// International textual data
#[derive(Clone, Debug, Default)]
pub struct Itxt {
    pub keyword: String,
    pub compressed: bool,
    pub compression_method: PngCompressionMethod,
    pub language: String,
    pub translated_keyword: String,
    pub compressed_string: Vec<u8>,
}

impl Itxt {
    /// Constructor
    pub fn new(
        keyword: &str,
        compression_method: Option<PngCompressionMethod>,
        language: &str,
        translated_keyword: &str,
        string: &str,
    ) -> Self {
        let mut compressed_string = Vec::new();
        if compression_method == Some(PngCompressionMethod::Zlib) {
            let mut encoder = ZlibEncoder::new(string.as_bytes(), Compression::best());
            let _ = encoder.read_to_end(&mut compressed_string);
        } else {
            compressed_string.extend(string.bytes());
        }

        Self {
            keyword: keyword.to_string(),
            compressed: compression_method.is_some(),
            compression_method: compression_method.unwrap_or_default(),
            language: language.to_string(),
            translated_keyword: translated_keyword.to_string(),
            compressed_string,
        }
    }

    /// Read contents from a stream
    pub fn from_stream<R>(
        stream: &mut R,
        length: u32,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = vec![0_u8; length as usize];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        let keyword_end = find_null(&data);
        let language_end = find_null(&data[keyword_end + 3..]) + keyword_end + 3;
        let tkeyword_end = find_null(&data[language_end + 1..]) + language_end + 1;

        Ok(Self {
            keyword: data[0..keyword_end].iter().map(|b| *b as char).collect(),
            compressed: data[keyword_end + 1] > 0,
            compression_method: data[keyword_end + 2].try_into().map_err(to_io_error)?,
            language: data[keyword_end + 3..language_end]
                .iter()
                .map(|b| *b as char)
                .collect(),
            translated_keyword: String::from_utf8(data[language_end + 1..tkeyword_end].to_vec())
                .map_err(to_io_error)?,
            compressed_string: data[tkeyword_end + 1..].to_vec(),
        })
    }

    /// Set the string
    pub fn set_string(&mut self, compression_method: Option<PngCompressionMethod>, string: &str) {
        let mut compressed_string = Vec::new();
        if compression_method == Some(PngCompressionMethod::Zlib) {
            let mut encoder = ZlibEncoder::new(string.as_bytes(), Compression::best());
            let _ = encoder.read_to_end(&mut compressed_string);
            self.compressed = true;
        } else {
            compressed_string.extend(string.bytes());
            self.compressed = false;
        }

        self.compression_method = compression_method.unwrap_or_default();
        self.compressed_string = compressed_string;
    }

    /// Decompress the compressed string in an iTXt chunk
    pub fn string(&self) -> Option<String> {
        if self.compressed {
            if self.compression_method == PngCompressionMethod::Zlib {
                let mut decoder = ZlibDecoder::new(self.compressed_string.as_slice());
                let mut out = String::new();
                if decoder.read_to_string(&mut out).is_ok() {
                    return Some(out);
                }
            }

            return None;
        }

        String::from_utf8(self.compressed_string.to_vec()).ok()
    }
}
