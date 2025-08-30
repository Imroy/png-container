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

use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};

/// All of the different file types based on PNG
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PNGFileType {
    /// Portable Network Graphics
    PNG,

    /// Multiple-image Network Graphics
    MNG,

    /// JPEG Network Graphics
    JNG,

    /// Animated Portable Network Graphics
    APNG,
}

/// Colour type of image
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGCompressionMethod {
    /// DEFLATE
    #[default]
    Zlib = 0,
}

/// Filter methods
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGFilterMethod {
    /// Adaptive filtering with five basic filter types
    #[default]
    Adaptive = 0,
}

/// Filter types
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PNGFilterType {
    None = 0,
    Sub,
    Up,
    Average,
    Paeth,
}

/// Interlacing methods
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
    Greyscale { value: u16 },

    TrueColour { red: u16, green: u16, blue: u16 },

    IndexedColour { values: Vec<u8> },
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
    Greyscale { value: u16 },

    TrueColour { red: u16, green: u16, blue: u16 },

    IndexedColour { index: u8 },
}

/// Unit type used in several chunks
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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

/// H.273 colour primaries
#[derive(Clone, Copy, Debug, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum ColourPrimaries {
    /// Rec. ITU-R BT.709-6\
    /// Rec. ITU-R BT.1361-0 conventional colour gamut\
    /// system and extended colour gamut system (historical)\
    /// IEC 61966-2-1 sRGB or sYCC\
    /// IEC 61966-2-4\
    /// SMPTE RP 177 Annex B
    BT709 = 1,

    /// Image characteristics are unknown or are determined by the application.
    Unspecified = 2,

    /// Rec. ITU-R BT.470-6 System M (historical)\
    /// United States National Television System Committee\
    /// 1953 Recommendation for transmission standards for color television\
    /// United States Federal Communications Commission (2003) Title 47 Code of Federal Regulations 73.682 (a) (20)
    SystemM = 4,

    /// Rec. ITU-R BT.470-6 System B, G (historical)\
    /// Rec. ITU-R BT.601-7 625\
    /// Rec. ITU-R BT.1358-0 625 (historical)\
    /// Rec. ITU-R BT.1700-0 625 PAL and 625 SECAM
    SystemBG = 5,

    /// Rec. ITU-R BT.601-7 525\
    /// Rec. ITU-R BT.1358-1 525 or 625 (historical)\
    /// Rec. ITU-R BT.1700-0 NTSC\
    /// SMPTE ST 170\
    /// (functionally the same as the value 7)
    BT601 = 6,

    /// SMPTE ST 240\
    /// (functionally the same as the value 6)
    ST240 = 7,

    /// Generic film (colour filters using Illuminant C)
    GenericFilm = 8,

    /// Rec. ITU-R BT.2020-2\
    /// Rec. ITU-R BT.2100-2
    BT2020 = 9,

    /// SMPTE ST 428-1\
    /// (CIE 1931 XYZ as in ISO/CIE 11664-1)
    ST428 = 10,

    /// SMPTE RP 431-2
    RP431 = 11,

    /// SMPTE EG 432-1
    EG432 = 12,

    /// No corresponding industry specification identified
    NoSpec = 22,

    /// For future use by ITU-T | ISO/IEC
    #[num_enum(catch_all)]
    Reserved(u8),
}

impl ColourPrimaries {
    /// Scaled red coordinates of the primary
    pub fn red_coords(self) -> (f64, f64) {
        match self {
            ColourPrimaries::BT709 => (0.64, 0.33),
            ColourPrimaries::SystemM => (0.67, 0.33),
            ColourPrimaries::SystemBG => (0.64, 0.33),
            ColourPrimaries::BT601 => (0.63, 0.34),
            ColourPrimaries::ST240 => (0.63, 0.34),
            ColourPrimaries::GenericFilm => (0.681, 0.319),
            ColourPrimaries::BT2020 => (0.708, 0.292),
            ColourPrimaries::ST428 => (1.0, 0.0),
            ColourPrimaries::RP431 => (0.68, 0.32),
            ColourPrimaries::EG432 => (0.68, 0.32),
            ColourPrimaries::NoSpec => (0.63, 0.34),

            ColourPrimaries::Unspecified | ColourPrimaries::Reserved(_) => (0.0, 0.0),
        }
    }

    /// Scaled green coordinates of the primary
    pub fn green_coords(self) -> (f64, f64) {
        match self {
            ColourPrimaries::BT709 => (0.3, 0.6),
            ColourPrimaries::SystemM => (0.21, 0.71),
            ColourPrimaries::SystemBG => (0.29, 0.60),
            ColourPrimaries::BT601 => (0.31, 0.595),
            ColourPrimaries::ST240 => (0.31, 0.595),
            ColourPrimaries::GenericFilm => (0.243, 0.692),
            ColourPrimaries::BT2020 => (0.17, 0.797),
            ColourPrimaries::ST428 => (0.0, 1.0),
            ColourPrimaries::RP431 => (0.265, 0.69),
            ColourPrimaries::EG432 => (0.265, 0.69),
            ColourPrimaries::NoSpec => (0.295, 0.605),

            ColourPrimaries::Unspecified | ColourPrimaries::Reserved(_) => (0.0, 0.0),
        }
    }

    /// Scaled blue coordinates of the primary
    pub fn blue_coords(self) -> (f64, f64) {
        match self {
            ColourPrimaries::BT709 => (0.15, 0.06),
            ColourPrimaries::SystemM => (0.14, 0.08),
            ColourPrimaries::SystemBG => (0.15, 0.06),
            ColourPrimaries::BT601 => (0.155, 0.07),
            ColourPrimaries::ST240 => (0.155, 0.07),
            ColourPrimaries::GenericFilm => (0.145, 0.049),
            ColourPrimaries::BT2020 => (0.131, 0.046),
            ColourPrimaries::ST428 => (0.0, 0.0),
            ColourPrimaries::RP431 => (0.15, 0.06),
            ColourPrimaries::EG432 => (0.15, 0.06),
            ColourPrimaries::NoSpec => (0.155, 0.077),

            ColourPrimaries::Unspecified | ColourPrimaries::Reserved(_) => (0.0, 0.0),
        }
    }

    /// Scaled white coordinates of the primary
    pub fn white_coords(self) -> (f64, f64) {
        match self {
            ColourPrimaries::BT709 => (0.3127, 0.329),
            ColourPrimaries::SystemM => (0.31, 0.316),
            ColourPrimaries::SystemBG => (0.3127, 0.329),
            ColourPrimaries::BT601 => (0.3127, 0.329),
            ColourPrimaries::ST240 => (0.3127, 0.329),
            ColourPrimaries::GenericFilm => (0.31, 0.316),
            ColourPrimaries::BT2020 => (0.3127, 0.329),
            ColourPrimaries::ST428 => (1.0 / 3.0, 1.0 / 3.0),
            ColourPrimaries::RP431 => (0.314, 0.351),
            ColourPrimaries::EG432 => (0.3127, 0.329),
            ColourPrimaries::NoSpec => (0.3127, 0.329),

            ColourPrimaries::Unspecified | ColourPrimaries::Reserved(_) => (0.0, 0.0),
        }
    }

}

/// H.273 transfer functions
#[derive(Clone, Copy, Debug, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum TransferFunction {
    /// Rec. ITU-R BT.709-6\
    /// Rec. ITU-R BT.1361-0 conventional colour gamut system (historical)\
    /// (functionally the same as the values 6, 14 and 15)
    BT709 = 1,

    /// Image characteristics are unknown or are determined by the application.
    Unspecified = 2,

    /// Rec. ITU-R BT.470-6 System M (historical)\
    /// United States National Television System Committee 1953\
    /// Recommendation for transmission standards for color television\
    /// United States Federal Communications Commission (2003) Title 47 Code of\
    /// Federal Regulations 73.682 (a) (20) Rec. ITU-R BT.1700-0 625 PAL and 625 SECAM
    SystemM = 4,

    /// Rec. ITU-R BT.470-6 System B, G (historical)
    SystemBG = 5,

    /// Rec. ITU-R BT.601-7 525 or 625\
    /// Rec. ITU-R BT.1358-1 525 or 625 (historical)\
    /// Rec. ITU-R BT.1700-0 NTSC\
    /// SMPTE ST 170\
    /// (functionally the same as the values 1, 14 and 15)
    BT601 = 6,

    /// SMPTE ST 240
    ST240 = 7,

    /// Linear transfer characteristics
    Linear = 8,

    /// Logarithmic transfer characteristic (100:1 range)
    Log100 = 9,

    /// Logarithmic transfer characteristic (100 * Sqrt( 10 ) : 1 range)
    Log316 = 10,

    /// IEC 61966-2-4
    IEC61966 = 11,

    /// Rec. ITU-R BT.1361-0 extended colour gamut system (historical)
    BT1361 = 12,

    /// IEC 61966-2-1 sRGB (with MatrixCoefficients equal to 0)\
    /// IEC 61966-2-1 sYCC (with MatrixCoefficients equal to 5)
    SrgbSycc = 13,

    /// Rec. ITU-R BT.2020-2 (10-bit system)\
    /// (functionally the same as the values 1, 6 and 15)
    BT2020_10b = 14,

    /// Rec. ITU-R BT.2020-2 (12-bit system)\
    /// (functionally the same as the values 1, 6 and 14)
    BT2020_12b = 15,

    /// SMPTE ST 2084 for 10-, 12-, 14- and 16-bit systems\
    /// Rec. ITU-R BT.2100-2 perceptual quantization (PQ) system
    ST2084 = 16,

    /// SMPTE ST 428-1
    ST428 = 17,

    /// ARIB STD-B67\
    /// Rec. ITU-R BT.2100-2 hybrid log-gamma (HLG) system
    HLG = 18,

    /// For future use by ITU-T | ISO/IEC
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// H.273 matrix coefficients
#[derive(Clone, Copy, Debug, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum MatrixCoefficients {
    /// The identity matrix.\
    /// Typically used for GBR (often referred to as RGB); however, may also be used for YZX (often referred to as XYZ);\
    /// IEC 61966-2-1 sRGB\
    /// SMPTE ST 428-1
    Identity = 0,

    /// Rec. ITU-R BT.709-6\
    /// Rec. ITU-R BT.1361-0 conventional colour gamut system and extended colour gamut system (historical)\
    /// IEC 61966-2-4 xvYCC709\
    /// SMPTE RP 177 Annex B
    BT709 = 1,

    /// Image characteristics are unknown or are determined by the application
    Unspecified = 2,

    /// United States Federal Communications Commission (2003) Title 47 Code of
    /// Federal Regulations 73.682 (a) (20)
    Title47 = 4,

    /// Rec. ITU-R BT.470-6 System B, G (historical)\
    /// Rec. ITU-R BT.601-7 625\
    /// Rec. ITU-R BT.1358-0 625 (historical)\
    /// Rec. ITU-R BT.1700-0 625 PAL and 625 SECAM\
    /// IEC 61966-2-1 sYCC\
    /// IEC 61966-2-4 xvYCC601\
    /// (functionally the same as the value 6)
    SystemBG = 5,

    /// Rec. ITU-R BT.601-7 525\
    /// Rec. ITU-R BT.1358-1 525 or 625 (historical)\
    /// Rec. ITU-R BT.1700-0 NTSC\
    /// SMPTE ST 170\
    /// (functionally the same as the value 5)
    BT601 = 6,

    /// SMPTE ST 240
    ST240 = 7,

    YCgCo = 8,

    /// Rec. ITU-R BT.2020-2 (non-constant luminance)\
    /// Rec. ITU-R BT.2100-2 Y′CbCr
    Bt2020NonConstLum = 9,

    /// Rec. ITU-R BT.2020-2 (constant luminance)
    Bt2020ConstLum = 10,

    /// SMPTE ST 2085
    ST2085 = 11,

    /// Chromaticity-derived non-constant luminance system
    ChromaNonConstLum = 12,

    /// Chromaticity-derived constant luminance system
    ChromaConstLum = 13,

    /// Rec. ITU-R BT.2100-2 ICTCP
    BT2100 = 14,

    /// Colour representation developed in SMPTE as IPT-PQ-C2.
    IptPqC2 = 15,

    YCgCoRe = 16,

    YCgCoRo = 17,

    /// For future use by ITU-T | ISO/IEC
    #[num_enum(catch_all)]
    Reserved(u8),
}

/// Disposal operators in the "fcTL" chunk
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum APNGDisposalOperator {
    None,
    Background,
    Previous,
}

/// Blend operators in the "fcTL" chunk
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum APNGBlendOperator {
    Source,
    Over,
}

/// Colour type of JNG image
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum JNGImageSampleDepth {
    Depth8 = 8,

    Depth12 = 12,

    Depth8And12 = 20,
}

/// JNG image and alpha compression type
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum JNGCompressionType {
    /// PNG greyscale
    PNGGreyscale = 0,

    /// Huffman-coded baseline JPEG
    HuffmanBaseline = 8,
}

/// JNG alpha sample depth
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum JNGInterlaceMethod {
    SequentialJPEG = 0,

    ProgressiveJPEG = 8,
}
