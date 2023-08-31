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

/*! PNG types
 */

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// All of the different file types based on PNG
#[derive(Copy, Clone, Debug)]
pub enum PNGFileType {
    /// Portable Network Graphics
    PNG,

    /// Multiple-image Network Graphics
    MNG,

    /// JPEG Network Graphics
    JNG,

    /// Animated Portable Network Graphics
    APNG
}

/// Colour type of image
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGColourType {
    /// Greyscale image - allowed depths of 1, 2, 4, 8, or 16 bits per component
    Greyscale = 0,

    /// RGB colour image - allowed depths of 8 or 16 bits per component
    TrueColour = 2,

    /// Indexed colour image - allowed depths of 1, 2, 4, or 8 bits per index
    IndexedColour,

    /// Greyscale image with alpha - allowed depths of 8 or 16 bits per component
    GreyscaleAlpha,

    /// RGB colour image with alpha - allowed depths of 8 or 16 bits per component
    TrueColourAlpha = 6,

}

/// Compression method(s)
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGCompressionMethod {
    /// DEFLATE
    Zlib = 0,

}

/// Filter methods
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGFilterMethod {
    /// Adaptive filtering with five basic filter types
    Adaptive = 0,

}

/// Filter types
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGFilterType {
    None = 0,
    Sub,
    Up,
    Average,
    Paeth,

}

/// Interlacing methods
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGInterlaceMethod {
    /// No interlacing
    None = 0,

    /// Adam7 interlacing
    Adam7,

}

/// Palette entry for for PLTE chunk
#[derive(Clone, Debug)]
pub struct PNGPaletteEntry {
    pub red: u8,
    pub green: u8,
    pub blue: u8,

}


/// Contents of tRNS chunk
#[derive(Clone, Debug)]
pub enum PNGtRNSType {
    Greyscale {
        value: u16,
    },

    TrueColour {
        red: u16,
        green: u16,
        blue: u16,
    },

    IndexedColour {
        values: Vec<u8>,
    },

}


/// Contents of sBIT chunk
#[derive(Copy, Clone, Debug)]
pub enum PNGsBITType {
    Greyscale {
        grey_bits: u8,
    },

    Colour {
        red_bits: u8,
        green_bits: u8,
        blue_bits: u8,
    },

    GreyscaleAlpha {
        grey_bits: u8,
        alpha_bits: u8,
    },

    TrueColourAlpha {
        red_bits: u8,
        green_bits: u8,
        blue_bits: u8,
        alpha_bits: u8,
    },

}


/// ICC rendering intent
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGRenderingIntent {
    Perceptual = 0,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

/// Contents of bKGD chunk
#[derive(Copy, Clone, Debug)]
pub enum PNGbKGDType {
    Greyscale {
        value: u16,
    },

    TrueColour {
        red: u16,
        green: u16,
        blue: u16,
    },

    IndexedColour {
        index: u8,
    },

}


/// Unit type used in several chunks
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGUnitType {
    Unknown = 0,

    Metre = 1,

}

/// Entry for the suggested palette "sPLT" chunk
///
/// When depth=8, the red, green, blue, and alpha fields will actually be unscaled u8 values.
#[derive(Copy, Clone, Debug)]
pub struct PNGSuggestedPaletteEntry {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub alpha: u16,
    pub frequency: u16,
}


/// Disposal operators in the "fcTL" chunk
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum APNGDisposalOperator {
    None,
    Background,
    Previous,
}

/// Blend operators in the "fcTL" chunk
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum APNGBlendOperator {
    Source,
    Over,
}

/// Colour type of JNG image
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum JNGColourType {
    Greyscale = 8,

    Colour = 10,

    /// Greyscale with alpha channel
    GreyscaleAlpha = 12,

    /// Colour with alpha channel
    ColourAlpha = 14,

}

/// JNG image sample depth
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum JNGImageSampleDepth {
    Depth8 = 8,

    Depth12 = 12,

    Depth8And12 = 20,

}

/// JNG image and alpha compression type
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum JNGCompressionType {
    /// PNG greyscale
    PNGGreyscale = 0,

    /// Huffman-coded baseline JPEG
    HuffmanBaseline = 8,

}

/// JNG alpha sample depth
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum JNGAlphaSampleDepth {
    Depth0 = 0,
    Depth1 = 1,
    Depth2 = 2,
    Depth4 = 4,
    Depth8 = 8,
    Depth16 = 16,

}

/// JNG image and alpha interlace type
#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum JNGInterlaceMethod {
    SequentialJPEG = 0,

    ProgressiveJPEG = 8,
}
