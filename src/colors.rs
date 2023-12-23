use egui::Color32;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Sunset,
    Desert,
    Harlequin, // New Palette Theme
    Gentle,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", *self)
    }
}

impl Theme {
    pub fn colors(&self) -> &[Color32; 12] {
        match *self {
            Theme::Sunset => &SUNSET_COLORS,
            Theme::Desert => &DESERT_COLORS,
            Theme::Harlequin => &HARLEQUIN_COLORS, // New Palette match arm
            Theme::Gentle => &GENTLE_COLORS,       // New Palette match arm
        }
    }
}

// Sunset Palette
pub const SUNSET_COLORS: [Color32; 12] = [
    Color32::from_rgb(252, 94, 77),   // Deep Sunset
    Color32::from_rgb(252, 163, 17),  // Golden Hour
    Color32::from_rgb(107, 78, 113),  // Dusk Purple
    Color32::from_rgb(50, 115, 220),  // Twilight
    Color32::from_rgb(3, 37, 108),    // Midnight Blue
    Color32::from_rgb(233, 68, 172),  // Pink Horizon
    Color32::from_rgb(252, 190, 50),  // Golden Sky
    Color32::from_rgb(64, 63, 151),   // Early Night
    Color32::from_rgb(252, 118, 106), // Sunset Glow
    Color32::from_rgb(242, 85, 96),   // Red Horizon
    Color32::from_rgb(76, 40, 130),   // Purple Cloud
    Color32::from_rgb(254, 207, 101), // Soft Orange
];

// Desert Palette
pub const DESERT_COLORS: [Color32; 12] = [
    Color32::from_rgb(254, 221, 170), // Sandstone
    Color32::from_rgb(87, 115, 34),   // Cactus
    Color32::from_rgb(194, 58, 22),   // Clay
    Color32::from_rgb(255, 104, 31),  // Sunset Orange
    Color32::from_rgb(55, 71, 79),    // Twilight Cactus
    Color32::from_rgb(255, 228, 196), // Bisque
    Color32::from_rgb(255, 222, 173), // Navajo White
    Color32::from_rgb(210, 180, 140), // Tan
    Color32::from_rgb(218, 165, 32),  // Golden Rod
    Color32::from_rgb(184, 134, 11),  // Dark Golden Rod
    Color32::from_rgb(244, 164, 96),  // Sandy Brown
    Color32::from_rgb(210, 105, 30),  // Chocolate
];

pub const HARLEQUIN_COLORS: [Color32; 12] = [
    Color32::from_rgb(2, 132, 130),   // Turquoise
    Color32::from_rgb(255, 0, 0),     // Red
    Color32::from_rgb(255, 165, 0),   // Orange
    Color32::from_rgb(255, 255, 0),   // Yellow
    Color32::from_rgb(0, 128, 0),     // Green
    Color32::from_rgb(0, 0, 255),     // Blue
    Color32::from_rgb(128, 0, 128),   // Purple
    Color32::from_rgb(255, 192, 203), // Pink
    Color32::from_rgb(128, 128, 0),   // Olive
    Color32::from_rgb(0, 255, 255),   // Cyan
    Color32::from_rgb(165, 42, 42),   // Brown
    Color32::from_rgb(255, 215, 0),   // Gold
];

pub const GENTLE_COLORS: [Color32; 12] = [
    Color32::from_rgb(166, 206, 227),
    Color32::from_rgb(31, 120, 180),
    Color32::from_rgb(178, 223, 138),
    Color32::from_rgb(51, 160, 44),
    Color32::from_rgb(251, 154, 153),
    Color32::from_rgb(227, 26, 28),
    Color32::from_rgb(253, 191, 111),
    Color32::from_rgb(255, 127, 0),
    Color32::from_rgb(202, 178, 214),
    Color32::from_rgb(106, 61, 154),
    Color32::from_rgb(255, 255, 153),
    Color32::from_rgb(177, 89, 40),
];
