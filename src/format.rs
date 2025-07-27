use strum::FromRepr;

#[derive(Debug, Clone)]
pub enum TexFormatFamily {
    Astc {
        typeless: bool,
        unorm: bool,
        srgb: bool,
    },
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromRepr)]
#[cfg_attr(debug_assertions, derive(strum::IntoStaticStr, strum::EnumIter))]
pub enum TexFormat {
    A8Unorm = 0x41,
    Astc10x10Typeless = 0x422,
    Astc10x10Unorm = 0x423,
    Astc10x10UnormSrgb = 0x424,
    Astc10x5Typeless = 0x419,
    Astc10x5Unorm = 0x41A,
    Astc10x5UnormSrgb = 0x41B,
    Astc10x6Typeless = 0x41C,
    Astc10x6Unorm = 0x41D,
    Astc10x6UnormSrgb = 0x41E,
    Astc10x8Typeless = 0x41F,
    Astc10x8Unorm = 0x420,
    Astc10x8UnormSrgb = 0x421,
    Astc12x10Typeless = 0x425,
    Astc12x10Unorm = 0x426,
    Astc12x10UnormSrgb = 0x427,
    Astc12x12Typeless = 0x428,
    Astc12x12Unorm = 0x429,
    Astc12x12UnormSrgb = 0x42A,
    Astc4x4Typeless = 0x401,
    Astc4x4Unorm = 0x402,
    Astc4x4UnormSrgb = 0x403,
    Astc5x4Typeless = 0x404,
    Astc5x4Unorm = 0x405,
    Astc5x4UnormSrgb = 0x406,
    Astc5x5Typeless = 0x407,
    Astc5x5Unorm = 0x408,
    Astc5x5UnormSrgb = 0x409,
    Astc6x5Typeless = 0x40A,
    Astc6x5Unorm = 0x40B,
    Astc6x5UnormSrgb = 0x40C,
    Astc6x6Typeless = 0x40D,
    Astc6x6Unorm = 0x40E,
    Astc6x6UnormSrgb = 0x40F,
    Astc8x5Typeless = 0x410,
    Astc8x5Unorm = 0x411,
    Astc8x5UnormSrgb = 0x412,
    Astc8x6Typeless = 0x413,
    Astc8x6Unorm = 0x414,
    Astc8x6UnormSrgb = 0x415,
    Astc8x8Typeless = 0x416,
    Astc8x8Unorm = 0x417,
    Astc8x8UnormSrgb = 0x418,
    B5G5R5A1Unorm = 0x56,
    B5G6R5Unorm = 0x55,
    B8G8R8A8Typeless = 0x5A,
    B8G8R8A8Unorm = 0x57,
    B8G8R8A8UnormSrgb = 0x5B,
    B8G8R8X8Typeless = 0x5C,
    B8G8R8X8Unorm = 0x58,
    B8G8R8X8UnormSrgb = 0x5D,
    Bc1Typeless = 0x46,
    Bc1Unorm = 0x47,
    Bc1UnormSrgb = 0x48,
    Bc2Typeless = 0x49,
    Bc2Unorm = 0x4A,
    Bc2UnormSrgb = 0x4B,
    Bc3Typeless = 0x4C,
    Bc3Unorm = 0x4D,
    Bc3UnormSrgb = 0x4E,
    Bc4Typeless = 0x4F,
    Bc4Unorm = 0x50,
    Bc4Snorm = 0x51,
    Bc5Typeless = 0x52,
    Bc5Unorm = 0x53,
    Bc5Snorm = 0x54,
    Bc6hTypeless = 0x5E,
    Bc6hUF16 = 0x5F,
    Bc6hSF16 = 0x60,
    Bc7Typeless = 0x61,
    Bc7Unorm = 0x62,
    Bc7UnormSrgb = 0x63,
    D16Unorm = 0x37,
    D24UnormS8Uint = 0x2D,
    D32Float = 0x28,
    D32FloatS8X24Uint = 0x14,
    ForceUint = 0x7FFFFFFF,
    G8R8G8B8Unorm = 0x45,
    R10G10B10A2Typeless = 0x17,
    R10G10B10A2Uint = 0x19,
    R10G10B10A2Unorm = 0x18,
    R10G10B10xrBiasA2Unorm = 0x59,
    R11G11B10Float = 0x1A,
    R16Float = 0x36,
    R16G16B16A16Float = 0xA,
    R16G16B16A16Sint = 0xE,
    R16G16B16A16Snorm = 0xD,
    R16G16B16A16Typeless = 0x9,
    R16G16B16A16Uint = 0xC,
    R16G16B16A16Unorm = 0xB,
    R16G16Float = 0x22,
    R16G16Sint = 0x26,
    R16G16Snorm = 0x25,
    R16G16Typeless = 0x21,
    R16G16Uint = 0x24,
    R16G16Unorm = 0x23,
    R16Sint = 0x3B,
    R16Snorm = 0x3A,
    R16Typeless = 0x35,
    R16Uint = 0x39,
    R16Unorm = 0x38,
    R1Unorm = 0x42,
    R24G8Typeless = 0x2C,
    R24UnormX8Typeless = 0x2E,
    R32Float = 0x29,
    R32FloatX8X24Typeless = 0x15,
    R32G32B32A32Float = 0x2,
    R32G32B32A32Sint = 0x4,
    R32G32B32A32Typeless = 0x1,
    R32G32B32A32Uint = 0x3,
    R32G32B32Float = 0x6,
    R32G32B32Sint = 0x8,
    R32G32B32Typeless = 0x5,
    R32G32B32Uint = 0x7,
    R32G32Float = 0x10,
    R32G32Sint = 0x12,
    R32G32Typeless = 0xF,
    R32G32Uint = 0x11,
    R32G8X24Typeless = 0x13,
    R32Sint = 0x2B,
    R32Typeless = 0x27,
    R32Uint = 0x2A,
    R8G8B8A8Sint = 0x20,
    R8G8B8A8Snorm = 0x1F,
    R8G8B8A8Typeless = 0x1B,
    R8G8B8A8Uint = 0x1E,
    R8G8B8A8Unorm = 0x1C,
    R8G8B8A8UnormSrgb = 0x1D,
    R8G8B8G8Unorm = 0x44,
    R8G8Sint = 0x34,
    R8G8Snorm = 0x33,
    R8G8Typeless = 0x30,
    R8G8Uint = 0x32,
    R8G8Unorm = 0x31,
    R8Sint = 0x40,
    R8Snorm = 0x3F,
    R8Typeless = 0x3C,
    R8Uint = 0x3E,
    R8Unorm = 0x3D,
    R9G9B9E5Sharedexp = 0x43,
    ViaExtension = 0x400,
    X24TypelessG8Uint = 0x2F,
    X32TypelessG8X24Uint = 0x16,
}

impl TexFormat {
    pub fn is_astc(&self) -> bool {
        (*self) as u32 & 0x400 != 0
    }

    pub fn is_bc(&self) -> bool {
        let val = (*self) as u32;
        (0x46..=0x63).contains(&val)
    }

    pub fn is_rgb(&self) -> bool {
        matches!(
            self,
            TexFormat::A8Unorm
                | TexFormat::B5G5R5A1Unorm
                | TexFormat::B5G6R5Unorm
                | TexFormat::B8G8R8A8Typeless
                | TexFormat::B8G8R8A8Unorm
                | TexFormat::B8G8R8A8UnormSrgb
                | TexFormat::B8G8R8X8Typeless
                | TexFormat::B8G8R8X8Unorm
                | TexFormat::B8G8R8X8UnormSrgb
                | TexFormat::G8R8G8B8Unorm
                | TexFormat::R10G10B10A2Typeless
                | TexFormat::R10G10B10A2Uint
                | TexFormat::R10G10B10A2Unorm
                | TexFormat::R10G10B10xrBiasA2Unorm
                | TexFormat::R11G11B10Float
                | TexFormat::R16Float
                | TexFormat::R16G16B16A16Float
                | TexFormat::R16G16B16A16Sint
                | TexFormat::R16G16B16A16Snorm
                | TexFormat::R16G16B16A16Typeless
                | TexFormat::R16G16B16A16Uint
                | TexFormat::R16G16B16A16Unorm
                | TexFormat::R16G16Float
                | TexFormat::R16G16Sint
                | TexFormat::R16G16Snorm
                | TexFormat::R16G16Typeless
                | TexFormat::R16G16Uint
                | TexFormat::R16G16Unorm
                | TexFormat::R16Sint
                | TexFormat::R16Snorm
                | TexFormat::R16Typeless
                | TexFormat::R16Uint
                | TexFormat::R16Unorm
                | TexFormat::R1Unorm
                | TexFormat::R24G8Typeless
                | TexFormat::R24UnormX8Typeless
                | TexFormat::R32Float
                | TexFormat::R32FloatX8X24Typeless
                | TexFormat::R32G32B32A32Float
                | TexFormat::R32G32B32A32Sint
                | TexFormat::R32G32B32A32Typeless
                | TexFormat::R32G32B32A32Uint
                | TexFormat::R32G32B32Float
                | TexFormat::R32G32B32Sint
                | TexFormat::R32G32B32Typeless
                | TexFormat::R32G32B32Uint
                | TexFormat::R32G32Float
                | TexFormat::R32G32Sint
                | TexFormat::R32G32Typeless
                | TexFormat::R32G32Uint
                | TexFormat::R32G8X24Typeless
                | TexFormat::R32Sint
                | TexFormat::R32Typeless
                | TexFormat::R32Uint
                | TexFormat::R8G8B8A8Sint
                | TexFormat::R8G8B8A8Snorm
                | TexFormat::R8G8B8A8Typeless
                | TexFormat::R8G8B8A8Uint
                | TexFormat::R8G8B8A8Unorm
                | TexFormat::R8G8B8A8UnormSrgb
                | TexFormat::R8G8B8G8Unorm
                | TexFormat::R8G8Sint
                | TexFormat::R8G8Snorm
                | TexFormat::R8G8Typeless
                | TexFormat::R8G8Uint
                | TexFormat::R8G8Unorm
                | TexFormat::R8Sint
                | TexFormat::R8Snorm
                | TexFormat::R8Typeless
                | TexFormat::R8Uint
                | TexFormat::R8Unorm
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_type() {
        assert!(TexFormat::Astc10x10Typeless.is_astc());
        assert!(TexFormat::Astc4x4Typeless.is_astc());
        assert!(TexFormat::Astc6x6UnormSrgb.is_astc());

        assert!(TexFormat::Bc1Typeless.is_bc());
        assert!(TexFormat::Bc3Typeless.is_bc());
        assert!(TexFormat::Bc7Unorm.is_bc());

        assert!(TexFormat::R8G8B8G8Unorm.is_rgb());
        assert!(TexFormat::R16G16B16A16Sint.is_rgb());
        assert!(TexFormat::R16G16B16A16Snorm.is_rgb());
    }
}
