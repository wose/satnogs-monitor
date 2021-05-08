use lazy_static::lazy_static;
use tui::style::Color;

lazy_static! {
    pub static ref VIRIDIS: Vec<Color> = vec![
        Color::Rgb(0x44, 0x01, 0x54),
        Color::Rgb(0x44, 0x02, 0x55),
        Color::Rgb(0x44, 0x03, 0x57),
        Color::Rgb(0x45, 0x05, 0x58),
        Color::Rgb(0x45, 0x06, 0x5A),
        Color::Rgb(0x45, 0x08, 0x5B),
        Color::Rgb(0x46, 0x09, 0x5C),
        Color::Rgb(0x46, 0x0B, 0x5E),
        Color::Rgb(0x46, 0x0C, 0x5F),
        Color::Rgb(0x46, 0x0E, 0x61),
        Color::Rgb(0x47, 0x0F, 0x62),
        Color::Rgb(0x47, 0x11, 0x63),
        Color::Rgb(0x47, 0x12, 0x65),
        Color::Rgb(0x47, 0x14, 0x66),
        Color::Rgb(0x47, 0x15, 0x67),
        Color::Rgb(0x47, 0x16, 0x69),
        Color::Rgb(0x47, 0x18, 0x6A),
        Color::Rgb(0x48, 0x19, 0x6B),
        Color::Rgb(0x48, 0x1A, 0x6C),
        Color::Rgb(0x48, 0x1C, 0x6E),
        Color::Rgb(0x48, 0x1D, 0x6F),
        Color::Rgb(0x48, 0x1E, 0x70),
        Color::Rgb(0x48, 0x20, 0x71),
        Color::Rgb(0x48, 0x21, 0x72),
        Color::Rgb(0x48, 0x22, 0x73),
        Color::Rgb(0x48, 0x23, 0x74),
        Color::Rgb(0x47, 0x25, 0x75),
        Color::Rgb(0x47, 0x26, 0x76),
        Color::Rgb(0x47, 0x27, 0x77),
        Color::Rgb(0x47, 0x28, 0x78),
        Color::Rgb(0x47, 0x2A, 0x79),
        Color::Rgb(0x47, 0x2B, 0x7A),
        Color::Rgb(0x47, 0x2C, 0x7B),
        Color::Rgb(0x46, 0x2D, 0x7C),
        Color::Rgb(0x46, 0x2F, 0x7C),
        Color::Rgb(0x46, 0x30, 0x7D),
        Color::Rgb(0x46, 0x31, 0x7E),
        Color::Rgb(0x45, 0x32, 0x7F),
        Color::Rgb(0x45, 0x34, 0x7F),
        Color::Rgb(0x45, 0x35, 0x80),
        Color::Rgb(0x45, 0x36, 0x81),
        Color::Rgb(0x44, 0x37, 0x81),
        Color::Rgb(0x44, 0x39, 0x82),
        Color::Rgb(0x43, 0x3A, 0x83),
        Color::Rgb(0x43, 0x3B, 0x83),
        Color::Rgb(0x43, 0x3C, 0x84),
        Color::Rgb(0x42, 0x3D, 0x84),
        Color::Rgb(0x42, 0x3E, 0x85),
        Color::Rgb(0x42, 0x40, 0x85),
        Color::Rgb(0x41, 0x41, 0x86),
        Color::Rgb(0x41, 0x42, 0x86),
        Color::Rgb(0x40, 0x43, 0x87),
        Color::Rgb(0x40, 0x44, 0x87),
        Color::Rgb(0x3F, 0x45, 0x87),
        Color::Rgb(0x3F, 0x47, 0x88),
        Color::Rgb(0x3E, 0x48, 0x88),
        Color::Rgb(0x3E, 0x49, 0x89),
        Color::Rgb(0x3D, 0x4A, 0x89),
        Color::Rgb(0x3D, 0x4B, 0x89),
        Color::Rgb(0x3D, 0x4C, 0x89),
        Color::Rgb(0x3C, 0x4D, 0x8A),
        Color::Rgb(0x3C, 0x4E, 0x8A),
        Color::Rgb(0x3B, 0x50, 0x8A),
        Color::Rgb(0x3B, 0x51, 0x8A),
        Color::Rgb(0x3A, 0x52, 0x8B),
        Color::Rgb(0x3A, 0x53, 0x8B),
        Color::Rgb(0x39, 0x54, 0x8B),
        Color::Rgb(0x39, 0x55, 0x8B),
        Color::Rgb(0x38, 0x56, 0x8B),
        Color::Rgb(0x38, 0x57, 0x8C),
        Color::Rgb(0x37, 0x58, 0x8C),
        Color::Rgb(0x37, 0x59, 0x8C),
        Color::Rgb(0x36, 0x5A, 0x8C),
        Color::Rgb(0x36, 0x5B, 0x8C),
        Color::Rgb(0x35, 0x5C, 0x8C),
        Color::Rgb(0x35, 0x5D, 0x8C),
        Color::Rgb(0x34, 0x5E, 0x8D),
        Color::Rgb(0x34, 0x5F, 0x8D),
        Color::Rgb(0x33, 0x60, 0x8D),
        Color::Rgb(0x33, 0x61, 0x8D),
        Color::Rgb(0x32, 0x62, 0x8D),
        Color::Rgb(0x32, 0x63, 0x8D),
        Color::Rgb(0x31, 0x64, 0x8D),
        Color::Rgb(0x31, 0x65, 0x8D),
        Color::Rgb(0x31, 0x66, 0x8D),
        Color::Rgb(0x30, 0x67, 0x8D),
        Color::Rgb(0x30, 0x68, 0x8D),
        Color::Rgb(0x2F, 0x69, 0x8D),
        Color::Rgb(0x2F, 0x6A, 0x8D),
        Color::Rgb(0x2E, 0x6B, 0x8E),
        Color::Rgb(0x2E, 0x6C, 0x8E),
        Color::Rgb(0x2E, 0x6D, 0x8E),
        Color::Rgb(0x2D, 0x6E, 0x8E),
        Color::Rgb(0x2D, 0x6F, 0x8E),
        Color::Rgb(0x2C, 0x70, 0x8E),
        Color::Rgb(0x2C, 0x71, 0x8E),
        Color::Rgb(0x2C, 0x72, 0x8E),
        Color::Rgb(0x2B, 0x73, 0x8E),
        Color::Rgb(0x2B, 0x74, 0x8E),
        Color::Rgb(0x2A, 0x75, 0x8E),
        Color::Rgb(0x2A, 0x76, 0x8E),
        Color::Rgb(0x2A, 0x77, 0x8E),
        Color::Rgb(0x29, 0x78, 0x8E),
        Color::Rgb(0x29, 0x79, 0x8E),
        Color::Rgb(0x28, 0x7A, 0x8E),
        Color::Rgb(0x28, 0x7A, 0x8E),
        Color::Rgb(0x28, 0x7B, 0x8E),
        Color::Rgb(0x27, 0x7C, 0x8E),
        Color::Rgb(0x27, 0x7D, 0x8E),
        Color::Rgb(0x27, 0x7E, 0x8E),
        Color::Rgb(0x26, 0x7F, 0x8E),
        Color::Rgb(0x26, 0x80, 0x8E),
        Color::Rgb(0x26, 0x81, 0x8E),
        Color::Rgb(0x25, 0x82, 0x8E),
        Color::Rgb(0x25, 0x83, 0x8D),
        Color::Rgb(0x24, 0x84, 0x8D),
        Color::Rgb(0x24, 0x85, 0x8D),
        Color::Rgb(0x24, 0x86, 0x8D),
        Color::Rgb(0x23, 0x87, 0x8D),
        Color::Rgb(0x23, 0x88, 0x8D),
        Color::Rgb(0x23, 0x89, 0x8D),
        Color::Rgb(0x22, 0x89, 0x8D),
        Color::Rgb(0x22, 0x8A, 0x8D),
        Color::Rgb(0x22, 0x8B, 0x8D),
        Color::Rgb(0x21, 0x8C, 0x8D),
        Color::Rgb(0x21, 0x8D, 0x8C),
        Color::Rgb(0x21, 0x8E, 0x8C),
        Color::Rgb(0x20, 0x8F, 0x8C),
        Color::Rgb(0x20, 0x90, 0x8C),
        Color::Rgb(0x20, 0x91, 0x8C),
        Color::Rgb(0x1F, 0x92, 0x8C),
        Color::Rgb(0x1F, 0x93, 0x8B),
        Color::Rgb(0x1F, 0x94, 0x8B),
        Color::Rgb(0x1F, 0x95, 0x8B),
        Color::Rgb(0x1F, 0x96, 0x8B),
        Color::Rgb(0x1E, 0x97, 0x8A),
        Color::Rgb(0x1E, 0x98, 0x8A),
        Color::Rgb(0x1E, 0x99, 0x8A),
        Color::Rgb(0x1E, 0x99, 0x8A),
        Color::Rgb(0x1E, 0x9A, 0x89),
        Color::Rgb(0x1E, 0x9B, 0x89),
        Color::Rgb(0x1E, 0x9C, 0x89),
        Color::Rgb(0x1E, 0x9D, 0x88),
        Color::Rgb(0x1E, 0x9E, 0x88),
        Color::Rgb(0x1E, 0x9F, 0x88),
        Color::Rgb(0x1E, 0xA0, 0x87),
        Color::Rgb(0x1F, 0xA1, 0x87),
        Color::Rgb(0x1F, 0xA2, 0x86),
        Color::Rgb(0x1F, 0xA3, 0x86),
        Color::Rgb(0x20, 0xA4, 0x85),
        Color::Rgb(0x20, 0xA5, 0x85),
        Color::Rgb(0x21, 0xA6, 0x85),
        Color::Rgb(0x21, 0xA7, 0x84),
        Color::Rgb(0x22, 0xA7, 0x84),
        Color::Rgb(0x23, 0xA8, 0x83),
        Color::Rgb(0x23, 0xA9, 0x82),
        Color::Rgb(0x24, 0xAA, 0x82),
        Color::Rgb(0x25, 0xAB, 0x81),
        Color::Rgb(0x26, 0xAC, 0x81),
        Color::Rgb(0x27, 0xAD, 0x80),
        Color::Rgb(0x28, 0xAE, 0x7F),
        Color::Rgb(0x29, 0xAF, 0x7F),
        Color::Rgb(0x2A, 0xB0, 0x7E),
        Color::Rgb(0x2B, 0xB1, 0x7D),
        Color::Rgb(0x2C, 0xB1, 0x7D),
        Color::Rgb(0x2E, 0xB2, 0x7C),
        Color::Rgb(0x2F, 0xB3, 0x7B),
        Color::Rgb(0x30, 0xB4, 0x7A),
        Color::Rgb(0x32, 0xB5, 0x7A),
        Color::Rgb(0x33, 0xB6, 0x79),
        Color::Rgb(0x35, 0xB7, 0x78),
        Color::Rgb(0x36, 0xB8, 0x77),
        Color::Rgb(0x38, 0xB9, 0x76),
        Color::Rgb(0x39, 0xB9, 0x76),
        Color::Rgb(0x3B, 0xBA, 0x75),
        Color::Rgb(0x3D, 0xBB, 0x74),
        Color::Rgb(0x3E, 0xBC, 0x73),
        Color::Rgb(0x40, 0xBD, 0x72),
        Color::Rgb(0x42, 0xBE, 0x71),
        Color::Rgb(0x44, 0xBE, 0x70),
        Color::Rgb(0x45, 0xBF, 0x6F),
        Color::Rgb(0x47, 0xC0, 0x6E),
        Color::Rgb(0x49, 0xC1, 0x6D),
        Color::Rgb(0x4B, 0xC2, 0x6C),
        Color::Rgb(0x4D, 0xC2, 0x6B),
        Color::Rgb(0x4F, 0xC3, 0x69),
        Color::Rgb(0x51, 0xC4, 0x68),
        Color::Rgb(0x53, 0xC5, 0x67),
        Color::Rgb(0x55, 0xC6, 0x66),
        Color::Rgb(0x57, 0xC6, 0x65),
        Color::Rgb(0x59, 0xC7, 0x64),
        Color::Rgb(0x5B, 0xC8, 0x62),
        Color::Rgb(0x5E, 0xC9, 0x61),
        Color::Rgb(0x60, 0xC9, 0x60),
        Color::Rgb(0x62, 0xCA, 0x5F),
        Color::Rgb(0x64, 0xCB, 0x5D),
        Color::Rgb(0x67, 0xCC, 0x5C),
        Color::Rgb(0x69, 0xCC, 0x5B),
        Color::Rgb(0x6B, 0xCD, 0x59),
        Color::Rgb(0x6D, 0xCE, 0x58),
        Color::Rgb(0x70, 0xCE, 0x56),
        Color::Rgb(0x72, 0xCF, 0x55),
        Color::Rgb(0x74, 0xD0, 0x54),
        Color::Rgb(0x77, 0xD0, 0x52),
        Color::Rgb(0x79, 0xD1, 0x51),
        Color::Rgb(0x7C, 0xD2, 0x4F),
        Color::Rgb(0x7E, 0xD2, 0x4E),
        Color::Rgb(0x81, 0xD3, 0x4C),
        Color::Rgb(0x83, 0xD3, 0x4B),
        Color::Rgb(0x86, 0xD4, 0x49),
        Color::Rgb(0x88, 0xD5, 0x47),
        Color::Rgb(0x8B, 0xD5, 0x46),
        Color::Rgb(0x8D, 0xD6, 0x44),
        Color::Rgb(0x90, 0xD6, 0x43),
        Color::Rgb(0x92, 0xD7, 0x41),
        Color::Rgb(0x95, 0xD7, 0x3F),
        Color::Rgb(0x97, 0xD8, 0x3E),
        Color::Rgb(0x9A, 0xD8, 0x3C),
        Color::Rgb(0x9D, 0xD9, 0x3A),
        Color::Rgb(0x9F, 0xD9, 0x38),
        Color::Rgb(0xA2, 0xDA, 0x37),
        Color::Rgb(0xA5, 0xDA, 0x35),
        Color::Rgb(0xA7, 0xDB, 0x33),
        Color::Rgb(0xAA, 0xDB, 0x32),
        Color::Rgb(0xAD, 0xDC, 0x30),
        Color::Rgb(0xAF, 0xDC, 0x2E),
        Color::Rgb(0xB2, 0xDD, 0x2C),
        Color::Rgb(0xB5, 0xDD, 0x2B),
        Color::Rgb(0xB7, 0xDD, 0x29),
        Color::Rgb(0xBA, 0xDE, 0x27),
        Color::Rgb(0xBD, 0xDE, 0x26),
        Color::Rgb(0xBF, 0xDF, 0x24),
        Color::Rgb(0xC2, 0xDF, 0x22),
        Color::Rgb(0xC5, 0xDF, 0x21),
        Color::Rgb(0xC7, 0xE0, 0x1F),
        Color::Rgb(0xCA, 0xE0, 0x1E),
        Color::Rgb(0xCD, 0xE0, 0x1D),
        Color::Rgb(0xCF, 0xE1, 0x1C),
        Color::Rgb(0xD2, 0xE1, 0x1B),
        Color::Rgb(0xD4, 0xE1, 0x1A),
        Color::Rgb(0xD7, 0xE2, 0x19),
        Color::Rgb(0xDA, 0xE2, 0x18),
        Color::Rgb(0xDC, 0xE2, 0x18),
        Color::Rgb(0xDF, 0xE3, 0x18),
        Color::Rgb(0xE1, 0xE3, 0x18),
        Color::Rgb(0xE4, 0xE3, 0x18),
        Color::Rgb(0xE7, 0xE4, 0x19),
        Color::Rgb(0xE9, 0xE4, 0x19),
        Color::Rgb(0xEC, 0xE4, 0x1A),
        Color::Rgb(0xEE, 0xE5, 0x1B),
        Color::Rgb(0xF1, 0xE5, 0x1C),
        Color::Rgb(0xF3, 0xE5, 0x1E),
        Color::Rgb(0xF6, 0xE6, 0x1F),
        Color::Rgb(0xF8, 0xE6, 0x21),
        Color::Rgb(0xFA, 0xE6, 0x22),
        Color::Rgb(0xFD, 0xE7, 0x24)
    ];
}
