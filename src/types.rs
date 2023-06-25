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
#[derive(Copy, Clone, Debug)]
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

impl TryFrom<u8> for PNGColourType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGColourType::Greyscale as u8 => Ok(PNGColourType::Greyscale),
            x if x == PNGColourType::TrueColour as u8 => Ok(PNGColourType::TrueColour),
            x if x == PNGColourType::IndexedColour as u8 => Ok(PNGColourType::IndexedColour),
            x if x == PNGColourType::GreyscaleAlpha as u8 => Ok(PNGColourType::GreyscaleAlpha),
            x if x == PNGColourType::TrueColourAlpha as u8 => Ok(PNGColourType::TrueColourAlpha),

            _ => Err(std::io::Error::other(format!("PNG: Invalid value of colour type ({})", val))),
        }
    }
}


/// Compression method(s)
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PNGCompressionMethod {
    /// DEFLATE
    Zlib = 0,

}

impl TryFrom<u8> for PNGCompressionMethod {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGCompressionMethod::Zlib as u8 => Ok(PNGCompressionMethod::Zlib),

            _ => Err(std::io::Error::other(format!("PNG: Invalid value of compression method ({})", val))),
        }
    }
}


/// Filter methods
#[derive(Copy, Clone, Debug)]
pub enum PNGFilterMethod {
    /// Adaptive filtering with five basic filter types
    Adaptive = 0,

}

impl TryFrom<u8> for PNGFilterMethod {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGFilterMethod::Adaptive as u8 => Ok(PNGFilterMethod::Adaptive),

            _ => Err(std::io::Error::other(format!("PNG: Invalid value of filter method ({})", val))),
        }
    }
}


/// Filter types
#[derive(Copy, Clone, Debug)]
pub enum PNGFilterType {
    None = 0,
    Sub,
    Up,
    Average,
    Paeth,

}

impl TryFrom<u8> for PNGFilterType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGFilterType::None as u8 => Ok(PNGFilterType::None),
            x if x == PNGFilterType::Sub as u8 => Ok(PNGFilterType::Sub),
            x if x == PNGFilterType::Up as u8 => Ok(PNGFilterType::Up),
            x if x == PNGFilterType::Average as u8 => Ok(PNGFilterType::Average),
            x if x == PNGFilterType::Paeth as u8 => Ok(PNGFilterType::Paeth),

            _ => Err(std::io::Error::other(format!("PNG: Invalid value of filter type ({})", val))),
        }
    }
}


/// Interlacing methods
#[derive(Copy, Clone, Debug)]
pub enum PNGInterlaceMethod {
    /// No interlacing
    None = 0,

    /// Adam7 interlacing
    Adam7,

}

impl TryFrom<u8> for PNGInterlaceMethod {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGInterlaceMethod::None as u8 => Ok(PNGInterlaceMethod::None),
            x if x == PNGInterlaceMethod::Adam7 as u8 => Ok(PNGInterlaceMethod::Adam7),

            _ => Err(std::io::Error::other(format!("PNG: Invalid value of interlace method ({})", val))),
        }
    }
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
#[derive(Copy, Clone, Debug)]
pub enum PNGRenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

impl TryFrom<u8> for PNGRenderingIntent {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGRenderingIntent::Perceptual as u8 => Ok(PNGRenderingIntent::Perceptual),
            x if x == PNGRenderingIntent::RelativeColorimetric as u8 => Ok(PNGRenderingIntent::RelativeColorimetric),
            x if x == PNGRenderingIntent::Saturation as u8 => Ok(PNGRenderingIntent::Saturation),
            x if x == PNGRenderingIntent::AbsoluteColorimetric as u8 => Ok(PNGRenderingIntent::AbsoluteColorimetric),

            _ => Err(std::io::Error::other(format!("PNG: Invalid value of rendering intent ({})", val))),
        }
    }

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
#[derive(Copy, Clone, Debug)]
pub enum PNGUnitType {
    Unknown = 0,

    Metre = 1,

}

impl TryFrom<u8> for PNGUnitType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == PNGUnitType::Unknown as u8 => Ok(PNGUnitType::Unknown),
            x if x == PNGUnitType::Metre as u8 => Ok(PNGUnitType::Metre),

            _ => Err(std::io::Error::other(format!("PNG: Invalid value of unit ({})", val))),
        }
    }

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
#[derive(Copy, Clone, Debug)]
pub enum APNGDisposalOperator {
    None,
    Background,
    Previous,
}

impl TryFrom<u8> for APNGDisposalOperator {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == APNGDisposalOperator::None as u8 => Ok(APNGDisposalOperator::None),
            x if x == APNGDisposalOperator::Background as u8 => Ok(APNGDisposalOperator::Background),
            x if x == APNGDisposalOperator::Previous as u8 => Ok(APNGDisposalOperator::Previous),

            _ => Err(std::io::Error::other(format!("APNG: Invalid value of disposal operator ({})", val))),
        }
    }

}


/// Blend operators in the "fcTL" chunk
#[derive(Copy, Clone, Debug)]
pub enum APNGBlendOperator {
    Source,
    Over,
}

impl TryFrom<u8> for APNGBlendOperator {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == APNGBlendOperator::Source as u8 => Ok(APNGBlendOperator::Source),
            x if x == APNGBlendOperator::Over as u8 => Ok(APNGBlendOperator::Over),

            _ => Err(std::io::Error::other(format!("APNG: Invalid value of blend operator ({})", val))),
        }
    }

}


/// Colour type of JNG image
#[derive(Copy, Clone, Debug)]
pub enum JNGColourType {
    Greyscale = 8,

    Colour = 10,

    /// Greyscale with alpha channel
    GreyscaleAlpha = 12,

    /// Colour with alpha channel
    ColourAlpha = 14,

}

impl TryFrom<u8> for JNGColourType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == JNGColourType::Greyscale as u8 => Ok(JNGColourType::Greyscale),
            x if x == JNGColourType::Colour as u8 => Ok(JNGColourType::Colour),
            x if x == JNGColourType::GreyscaleAlpha as u8 => Ok(JNGColourType::GreyscaleAlpha),
            x if x == JNGColourType::ColourAlpha as u8 => Ok(JNGColourType::ColourAlpha),

            _ => Err(std::io::Error::other(format!("JNG: Invalid value of colour type ({})", val))),
        }
    }

}


/// JNG image sample depth
#[derive(Copy, Clone, Debug)]
pub enum JNGImageSampleDepth {
    Depth8 = 8,

    Depth12 = 12,

    Depth8And12 = 20,

}

impl TryFrom<u8> for JNGImageSampleDepth {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == JNGImageSampleDepth::Depth8 as u8 => Ok(JNGImageSampleDepth::Depth8),
            x if x == JNGImageSampleDepth::Depth12 as u8 => Ok(JNGImageSampleDepth::Depth12),
            x if x == JNGImageSampleDepth::Depth8And12 as u8 => Ok(JNGImageSampleDepth::Depth8And12),

            _ => Err(std::io::Error::other(format!("JNG: Invalid value of image sample depth ({})", val))),
        }
    }

}


/// JNG image and alpha compression type
#[derive(Copy, Clone, Debug)]
pub enum JNGCompressionType {
    /// PNG greyscale
    PNGGreyscale = 0,

    /// Huffman-coded baseline JPEG
    HuffmanBaseline = 8,

}

impl TryFrom<u8> for JNGCompressionType {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == JNGCompressionType::PNGGreyscale as u8 => Ok(JNGCompressionType::PNGGreyscale),
            x if x == JNGCompressionType::HuffmanBaseline as u8 => Ok(JNGCompressionType::HuffmanBaseline),

            _ => Err(std::io::Error::other(format!("JNG: Invalid value of compression type ({})", val))),
        }
    }

}


/// JNG alpha sample depth
#[derive(Copy, Clone, Debug)]
pub enum JNGAlphaSampleDepth {
    Depth0 = 0,
    Depth1 = 1,
    Depth2 = 2,
    Depth4 = 4,
    Depth8 = 8,
    Depth16 = 16,

}

impl TryFrom<u8> for JNGAlphaSampleDepth {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == JNGAlphaSampleDepth::Depth0 as u8 => Ok(JNGAlphaSampleDepth::Depth0),
            x if x == JNGAlphaSampleDepth::Depth1 as u8 => Ok(JNGAlphaSampleDepth::Depth1),
            x if x == JNGAlphaSampleDepth::Depth2 as u8 => Ok(JNGAlphaSampleDepth::Depth2),
            x if x == JNGAlphaSampleDepth::Depth4 as u8 => Ok(JNGAlphaSampleDepth::Depth4),
            x if x == JNGAlphaSampleDepth::Depth8 as u8 => Ok(JNGAlphaSampleDepth::Depth8),
            x if x == JNGAlphaSampleDepth::Depth16 as u8 => Ok(JNGAlphaSampleDepth::Depth16),

            _ => Err(std::io::Error::other(format!("JNG: Invalid value of alpha sample depth ({})", val))),
        }
    }

}


/// JNG image and alpha interlace type
#[derive(Copy, Clone, Debug)]
pub enum JNGInterlaceMethod {
    SequentialJPEG = 0,

    ProgressiveJPEG = 8,
}

impl TryFrom<u8> for JNGInterlaceMethod {
    type Error = std::io::Error;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == JNGInterlaceMethod::SequentialJPEG as u8 => Ok(JNGInterlaceMethod::SequentialJPEG),
            x if x == JNGInterlaceMethod::ProgressiveJPEG as u8 => Ok(JNGInterlaceMethod::ProgressiveJPEG),

            _ => Err(std::io::Error::other(format!("JNG: Invalid value of interlace type ({})", val))),
        }
    }

}

