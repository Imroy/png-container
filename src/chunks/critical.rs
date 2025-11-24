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

//! Critical chunks (IHDR, PLTE)

use std::io::{Read, Write};

use crate::crc::*;
use crate::to_io_error;
use crate::types::*;

/// Image header
#[derive(Clone, Copy, Debug)]
pub struct Ihdr {
    /// Width of image in pixels
    pub width: u32,

    /// Height of image in pixels
    pub height: u32,

    /// Number of bits per sample
    pub bit_depth: u8,

    /// Colour type
    pub colour_type: PngColourType,

    /// Compression method
    pub compression_method: PngCompressionMethod,

    /// Filter method
    pub filter_method: PngFilterMethod,

    /// Interlace method
    pub interlace_method: PngInterlaceMethod,
}

impl Ihdr {
    pub(crate) const TYPE: [u8; 4] = *b"IHDR";
    pub(crate) const LENGTH: u32 = 13;

    /// Constructor
    pub fn new(
        width: u32,
        height: u32,
        bit_depth: u8,
        colour_type: PngColourType,
        interlace_method: PngInterlaceMethod,
    ) -> Self {
        Self {
            width,
            height,
            bit_depth,
            colour_type,
            compression_method: PngCompressionMethod::default(),
            filter_method: PngFilterMethod::default(),
            interlace_method,
        }
    }

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 13];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            width: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            height: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            bit_depth: data[8],
            colour_type: data[9].try_into().map_err(to_io_error)?,
            compression_method: data[10].try_into().map_err(to_io_error)?,
            filter_method: data[11].try_into().map_err(to_io_error)?,
            interlace_method: data[12].try_into().map_err(to_io_error)?,
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
        let width_bytes = self.width.to_be_bytes();
        stream.write_all(&width_bytes)?;

        let height_bytes = self.width.to_be_bytes();
        stream.write_all(&height_bytes)?;

        let rest_bytes = [
            self.bit_depth,
            self.colour_type.into(),
            self.compression_method.into(),
            self.filter_method.into(),
            self.interlace_method.into(),
        ];
        stream.write_all(&rest_bytes)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&width_bytes);
            data_crc.consume(&height_bytes);
            data_crc.consume(&rest_bytes);
        }

        Ok(())
    }

    /// Number of bits in a pixel
    pub fn pixel_bits(&self) -> u8 {
        self.colour_type.num_components() * self.bit_depth
    }

    /// Number of bytes in a line of image data, rounded up
    pub fn line_size(&self) -> usize {
        let bits = self.width as usize * self.pixel_bits() as usize;

        // Divide by 8, but round up
        if bits & 0x07 > 0 {
            (bits >> 3) + 1
        } else {
            bits >> 3
        }
    }
}

/// Palette
#[derive(Clone, Debug, Default)]
pub struct Plte(pub Vec<PngPaletteEntry>);

impl Plte {
    pub(crate) const TYPE: [u8; 4] = *b"PLTE";

    /// Constructor
    pub fn new(palette: &[PngPaletteEntry]) -> Self {
        Self(palette.to_vec())
    }

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
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

        Ok(Self(
            data.chunks(3)
                .map(|col| {
                    Ok(PngPaletteEntry {
                        red: col[0],
                        green: col[1],
                        blue: col[2],
                    })
                })
                .collect::<Result<Vec<_>, std::io::Error>>()?,
        ))
    }

    pub(crate) fn length(&self) -> u32 {
        self.0.len() as u32 * 3
    }

    pub(crate) fn write_contents<W>(
        &self,
        stream: &mut W,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let buf = self
            .0
            .iter()
            .flat_map(|col| [col.red, col.green, col.blue])
            .collect::<Vec<u8>>();
        stream.write_all(&buf)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&buf);
        }

        Ok(())
    }
}

/// JNG header
#[derive(Clone, Copy, Debug)]
pub struct Jhdr {
    /// Width of image in pixels
    pub width: u32,

    /// Height of image in pixels
    pub height: u32,

    /// Colour type
    pub colour_type: JngColourType,

    /// Image sample depth
    pub image_sample_depth: JngImageSampleDepth,

    /// Image compression method
    pub image_compression_method: JngCompressionType,

    /// Image interlace method
    pub image_interlace_method: JngInterlaceMethod,

    /// Alpha sample depth
    pub alpha_sample_depth: JngAlphaSampleDepth,

    /// Alpha compression method
    pub alpha_compression_method: JngCompressionType,

    /// Alpha channel filter method
    pub alpha_filter_method: PngFilterMethod,

    /// Alpha interlace method
    pub alpha_interlace_method: JngInterlaceMethod,
}

impl Jhdr {
    pub(crate) const TYPE: [u8; 4] = *b"Jhdr";
    pub(crate) const LENGTH: u32 = 16;

    /// Constructor
    pub fn new(
        width: u32,
        height: u32,
        colour_type: JngColourType,
        image_sample_depth: JngImageSampleDepth,
        image_compression_method: JngCompressionType,
        image_interlace_method: JngInterlaceMethod,
        alpha_sample_depth: JngAlphaSampleDepth,
        alpha_compression_method: JngCompressionType,
        alpha_filter_method: PngFilterMethod,
        alpha_interlace_method: JngInterlaceMethod,
    ) -> Self {
        Self {
            width,
            height,
            colour_type,
            image_sample_depth,
            image_compression_method,
            image_interlace_method,
            alpha_sample_depth,
            alpha_compression_method,
            alpha_filter_method,
            alpha_interlace_method,
        }
    }

    /// Read contents from a stream
    pub fn from_contents_stream<R>(
        stream: &mut R,
        data_crc: Option<&mut CRC>,
    ) -> std::io::Result<Self>
    where
        R: Read,
    {
        let mut data = [0_u8; 16];
        stream.read_exact(&mut data)?;
        if let Some(data_crc) = data_crc {
            data_crc.consume(&data);
        }

        Ok(Self {
            width: u32::from_be_bytes(data[0..4].try_into().map_err(to_io_error)?),
            height: u32::from_be_bytes(data[4..8].try_into().map_err(to_io_error)?),
            colour_type: data[8].try_into().map_err(to_io_error)?,
            image_sample_depth: data[9].try_into().map_err(to_io_error)?,
            image_compression_method: data[10].try_into().map_err(to_io_error)?,
            image_interlace_method: data[11].try_into().map_err(to_io_error)?,
            alpha_sample_depth: data[12].try_into().map_err(to_io_error)?,
            alpha_compression_method: data[13].try_into().map_err(to_io_error)?,
            alpha_filter_method: data[14].try_into().map_err(to_io_error)?,
            alpha_interlace_method: data[15].try_into().map_err(to_io_error)?,
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
        let width_bytes = self.width.to_be_bytes();
        stream.write_all(&width_bytes)?;

        let height_bytes = self.width.to_be_bytes();
        stream.write_all(&height_bytes)?;

        let rest_bytes = [
            self.colour_type.into(),
            self.image_sample_depth.into(),
            self.image_compression_method.into(),
            self.image_interlace_method.into(),
            self.alpha_sample_depth.into(),
            self.alpha_compression_method.into(),
            self.alpha_filter_method.into(),
            self.alpha_interlace_method.into(),
        ];
        stream.write_all(&rest_bytes)?;

        if let Some(data_crc) = data_crc {
            data_crc.consume(&width_bytes);
            data_crc.consume(&height_bytes);
            data_crc.consume(&rest_bytes);
        }

        Ok(())
    }
}
