use std::ops::{Add, AddAssign, Mul, MulAssign};

use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red channel. [0.0, 1.0]
    pub red: f32,
    /// Green channel. [0.0, 1.0]
    pub green: f32,
    /// Blue channel. [0.0, 1.0]
    pub blue: f32,
    /// Alpha channel. [0.0, 1.0]
    pub alpha: f32,
}

impl Color {
    /// <div style="background-color:rgb(94%, 97%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ALICE_BLUE: Color = Color::rgb(0.94, 0.97, 1.0);
    /// <div style="background-color:rgb(98%, 92%, 84%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ANTIQUE_WHITE: Color = Color::rgb(0.98, 0.92, 0.84);
    /// <div style="background-color:rgb(49%, 100%, 83%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const AQUAMARINE: Color = Color::rgb(0.49, 1.0, 0.83);
    /// <div style="background-color:rgb(94%, 100%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const AZURE: Color = Color::rgb(0.94, 1.0, 1.0);
    /// <div style="background-color:rgb(96%, 96%, 86%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BEIGE: Color = Color::rgb(0.96, 0.96, 0.86);
    /// <div style="background-color:rgb(100%, 89%, 77%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BISQUE: Color = Color::rgb(1.0, 0.89, 0.77);
    /// <div style="background-color:rgb(0%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    /// <div style="background-color:rgb(0%, 0%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    /// <div style="background-color:rgb(86%, 8%, 24%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const CRIMSON: Color = Color::rgb(0.86, 0.08, 0.24);
    /// <div style="background-color:rgb(0%, 100%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
    /// <div style="background-color:rgb(25%, 25%, 25%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const DARK_GRAY: Color = Color::rgb(0.25, 0.25, 0.25);
    /// <div style="background-color:rgb(0%, 50%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const DARK_GREEN: Color = Color::rgb(0.0, 0.5, 0.0);
    /// <div style="background-color:rgb(100%, 0%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const FUCHSIA: Color = Color::rgb(1.0, 0.0, 1.0);
    /// <div style="background-color:rgb(100%, 84%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const GOLD: Color = Color::rgb(1.0, 0.84, 0.0);
    /// <div style="background-color:rgb(50%, 50%, 50%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
    /// <div style="background-color:rgb(0%, 100%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    /// <div style="background-color:rgb(28%, 0%, 51%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const INDIGO: Color = Color::rgb(0.29, 0.0, 0.51);
    /// <div style="background-color:rgb(20%, 80%, 20%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const LIME_GREEN: Color = Color::rgb(0.2, 0.8, 0.2);
    /// <div style="background-color:rgb(50%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const MAROON: Color = Color::rgb(0.5, 0.0, 0.0);
    /// <div style="background-color:rgb(10%, 10%, 44%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const MIDNIGHT_BLUE: Color = Color::rgb(0.1, 0.1, 0.44);
    /// <div style="background-color:rgb(0%, 0%, 50%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const NAVY: Color = Color::rgb(0.0, 0.0, 0.5);
    /// <div style="background-color:rgba(0%, 0%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
    /// <div style="background-color:rgb(50%, 50%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const OLIVE: Color = Color::rgb(0.5, 0.5, 0.0);
    /// <div style="background-color:rgb(100%, 65%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ORANGE: Color = Color::rgb(1.0, 0.65, 0.0);
    /// <div style="background-color:rgb(100%, 27%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const ORANGE_RED: Color = Color::rgb(1.0, 0.27, 0.0);
    /// <div style="background-color:rgb(100%, 8%, 57%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const PINK: Color = Color::rgb(1.0, 0.08, 0.58);
    /// <div style="background-color:rgb(50%, 0%, 50%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const PURPLE: Color = Color::rgb(0.5, 0.0, 0.5);
    /// <div style="background-color:rgb(100%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    /// <div style="background-color:rgb(98%, 50%, 45%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const SALMON: Color = Color::rgb(0.98, 0.5, 0.45);
    /// <div style="background-color:rgb(18%, 55%, 34%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const SEA_GREEN: Color = Color::rgb(0.18, 0.55, 0.34);
    /// <div style="background-color:rgb(75%, 75%, 75%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const SILVER: Color = Color::rgb(0.75, 0.75, 0.75);
    /// <div style="background-color:rgb(0%, 50%, 50%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const TEAL: Color = Color::rgb(0.0, 0.5, 0.5);
    /// <div style="background-color:rgb(100%, 39%, 28%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const TOMATO: Color = Color::rgb(1.0, 0.39, 0.28);
    /// <div style="background-color:rgb(25%, 88%, 82%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const TURQUOISE: Color = Color::rgb(0.25, 0.88, 0.82);
    /// <div style="background-color:rgb(93%, 51%, 93%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const VIOLET: Color = Color::rgb(0.93, 0.51, 0.93);
    /// <div style="background-color:rgb(100%, 100%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    /// <div style="background-color:rgb(100%, 100%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    /// <div style="background-color:rgb(60%, 80%, 20%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const YELLOW_GREEN: Color = Color::rgb(0.6, 0.8, 0.2);

    /// New `Color` from sRGB colorspace.
    ///
    /// # Arguments
    ///
    /// * `r` - Red channel. [0.0, 1.0]
    /// * `g` - Green channel. [0.0, 1.0]
    /// * `b` - Blue channel. [0.0, 1.0]
    ///
    /// See also [`Color::rgba`], [`Color::rgb_u8`], [`Color::hex`].
    ///
    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
            alpha: 1.0,
        }
    }

    /// New `Color` from sRGB colorspace.
    ///
    /// # Arguments
    ///
    /// * `r` - Red channel. [0.0, 1.0]
    /// * `g` - Green channel. [0.0, 1.0]
    /// * `b` - Blue channel. [0.0, 1.0]
    /// * `a` - Alpha channel. [0.0, 1.0]
    ///
    /// See also [`Color::rgb`], [`Color::rgba_u8`], [`Color::hex`].
    ///
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        }
    }

    /// New `Color` from sRGB colorspace.
    ///
    /// # Arguments
    ///
    /// * `r` - Red channel. [0, 255]
    /// * `g` - Green channel. [0, 255]
    /// * `b` - Blue channel. [0, 255]
    ///
    /// See also [`Color::rgb`], [`Color::rgba_u8`], [`Color::hex`].
    ///
    pub fn rgb_u8(r: u8, g: u8, b: u8) -> Color {
        Color::rgba_u8(r, g, b, u8::MAX)
    }

    // Float operations in const fn are not stable yet
    // see https://github.com/rust-lang/rust/issues/57241
    /// New `Color` from sRGB colorspace.
    ///
    /// # Arguments
    ///
    /// * `r` - Red channel. [0, 255]
    /// * `g` - Green channel. [0, 255]
    /// * `b` - Blue channel. [0, 255]
    /// * `a` - Alpha channel. [0, 255]
    ///
    /// See also [`Color::rgba`], [`Color::rgb_u8`], [`Color::hex`].
    ///
    pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::new(
            r as f32 / u8::MAX as f32,
            g as f32 / u8::MAX as f32,
            b as f32 / u8::MAX as f32,
            a as f32 / u8::MAX as f32,
        )
    }

    /// Converts a `Color` to a `[f32; 4]` from sRGB colorspace
    pub fn as_rgba_f32(self: Color) -> [f32; 4] {
        let Color {
            red,
            green,
            blue,
            alpha,
        } = self;

        [red, green, blue, alpha]
    }

    /// Converts `Color` to a `u32` from sRGB colorspace.
    ///
    /// Maps the RGBA channels in RGBA order to a little-endian byte array (GPUs are little-endian).
    /// `A` will be the most significant byte and `R` the least significant.
    pub fn as_rgba_u32(self: Color) -> u32 {
        let Color {
            red,
            green,
            blue,
            alpha,
        } = self;

        u32::from_le_bytes([
            (red * 255.0) as u8,
            (green * 255.0) as u8,
            (blue * 255.0) as u8,
            (alpha * 255.0) as u8,
        ])
    }

    fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        let Color {
            red,
            green,
            blue,
            alpha,
        } = self;
        let rhs = rhs.as_rgba_f32();

        self.red += rhs[0];
        self.green += rhs[1];
        self.blue += rhs[2];
        self.alpha += rhs[3];
    }
}

impl Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        let Color {
            red,
            green,
            blue,
            alpha,
        } = self;

        Color {
            red: red + rhs.red,
            green: green + rhs.green,
            blue: blue + rhs.blue,
            alpha: alpha + rhs.alpha,
        }
    }
}

impl AddAssign<Vec4> for Color {
    fn add_assign(&mut self, rhs: Vec4) {
        let rhs: Color = rhs.into();
        *self += rhs;
    }
}

impl Add<Vec4> for Color {
    type Output = Color;

    fn add(self, rhs: Vec4) -> Self::Output {
        let rhs: Color = rhs.into();
        self + rhs
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        color.as_rgba_f32()
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Color::rgba(r, g, b, a)
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Color::rgb(r, g, b)
    }
}

impl From<Color> for Vec4 {
    fn from(color: Color) -> Self {
        let color: [f32; 4] = color.into();
        Vec4::new(color[0], color[1], color[2], color[3])
    }
}

impl From<Vec4> for Color {
    fn from(vec4: Vec4) -> Self {
        Color::rgba(vec4.x, vec4.y, vec4.z, vec4.w)
    }
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        let Color {
            red,
            green,
            blue,
            alpha,
        } = color;

        wgpu::Color {
            r: red as f64,
            g: green as f64,
            b: blue as f64,
            a: alpha as f64,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        let Color {
            red,
            green,
            blue,
            alpha,
        } = self;

        Color {
            red: red * rhs,
            green: green * rhs,
            blue: blue * rhs,
            alpha,
        }
    }
}

// impl MulAssign<f32> for Color {
//     fn mul_assign(&mut self, rhs: f32) {
//         match self {
//             Color::Rgba {
//                 red, green, blue, ..
//             }
//             | Color::RgbaLinear {
//                 red, green, blue, ..
//             } => {
//                 *red *= rhs;
//                 *green *= rhs;
//                 *blue *= rhs;
//             }
//             Color::Hsla {
//                 hue,
//                 saturation,
//                 lightness,
//                 ..
//             } => {
//                 *hue *= rhs;
//                 *saturation *= rhs;
//                 *lightness *= rhs;
//             }
//         }
//     }
// }

// impl Mul<Vec4> for Color {
//     type Output = Color;

//     fn mul(self, rhs: Vec4) -> Self::Output {
//         match self {
//             Color::Rgba {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             } => Color::Rgba {
//                 red: red * rhs.x,
//                 green: green * rhs.y,
//                 blue: blue * rhs.z,
//                 alpha: alpha * rhs.w,
//             },
//             Color::RgbaLinear {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             } => Color::RgbaLinear {
//                 red: red * rhs.x,
//                 green: green * rhs.y,
//                 blue: blue * rhs.z,
//                 alpha: alpha * rhs.w,
//             },
//             Color::Hsla {
//                 hue,
//                 saturation,
//                 lightness,
//                 alpha,
//             } => Color::Hsla {
//                 hue: hue * rhs.x,
//                 saturation: saturation * rhs.y,
//                 lightness: lightness * rhs.z,
//                 alpha: alpha * rhs.w,
//             },
//         }
//     }
// }

// impl MulAssign<Vec4> for Color {
//     fn mul_assign(&mut self, rhs: Vec4) {
//         match self {
//             Color::Rgba {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             }
//             | Color::RgbaLinear {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             } => {
//                 *red *= rhs.x;
//                 *green *= rhs.y;
//                 *blue *= rhs.z;
//                 *alpha *= rhs.w;
//             }
//             Color::Hsla {
//                 hue,
//                 saturation,
//                 lightness,
//                 alpha,
//             } => {
//                 *hue *= rhs.x;
//                 *saturation *= rhs.y;
//                 *lightness *= rhs.z;
//                 *alpha *= rhs.w;
//             }
//         }
//     }
// }

impl Mul<Vec3> for Color {
    type Output = Color;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let Color {
            red,
            green,
            blue,
            alpha,
        } = self;

        Color {
            red: red * rhs.x,
            green: green * rhs.y,
            blue: blue * rhs.z,
            alpha,
        }
    }
}

// impl MulAssign<Vec3> for Color {
//     fn mul_assign(&mut self, rhs: Vec3) {
//         match self {
//             Color::Rgba {
//                 red, green, blue, ..
//             }
//             | Color::RgbaLinear {
//                 red, green, blue, ..
//             } => {
//                 *red *= rhs.x;
//                 *green *= rhs.y;
//                 *blue *= rhs.z;
//             }
//             Color::Hsla {
//                 hue,
//                 saturation,
//                 lightness,
//                 ..
//             } => {
//                 *hue *= rhs.x;
//                 *saturation *= rhs.y;
//                 *lightness *= rhs.z;
//             }
//         }
//     }
// }

// impl Mul<[f32; 4]> for Color {
//     type Output = Color;

//     fn mul(self, rhs: [f32; 4]) -> Self::Output {
//         match self {
//             Color::Rgba {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             } => Color::Rgba {
//                 red: red * rhs[0],
//                 green: green * rhs[1],
//                 blue: blue * rhs[2],
//                 alpha: alpha * rhs[3],
//             },
//             Color::RgbaLinear {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             } => Color::RgbaLinear {
//                 red: red * rhs[0],
//                 green: green * rhs[1],
//                 blue: blue * rhs[2],
//                 alpha: alpha * rhs[3],
//             },
//             Color::Hsla {
//                 hue,
//                 saturation,
//                 lightness,
//                 alpha,
//             } => Color::Hsla {
//                 hue: hue * rhs[0],
//                 saturation: saturation * rhs[1],
//                 lightness: lightness * rhs[2],
//                 alpha: alpha * rhs[3],
//             },
//         }
//     }
// }

// impl MulAssign<[f32; 4]> for Color {
//     fn mul_assign(&mut self, rhs: [f32; 4]) {
//         match self {
//             Color::Rgba {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             }
//             | Color::RgbaLinear {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             } => {
//                 *red *= rhs[0];
//                 *green *= rhs[1];
//                 *blue *= rhs[2];
//                 *alpha *= rhs[3];
//             }
//             Color::Hsla {
//                 hue,
//                 saturation,
//                 lightness,
//                 alpha,
//             } => {
//                 *hue *= rhs[0];
//                 *saturation *= rhs[1];
//                 *lightness *= rhs[2];
//                 *alpha *= rhs[3];
//             }
//         }
//     }
// }

// impl Mul<[f32; 3]> for Color {
//     type Output = Color;

//     fn mul(self, rhs: [f32; 3]) -> Self::Output {
//         match self {
//             Color::Rgba {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             } => Color::Rgba {
//                 red: red * rhs[0],
//                 green: green * rhs[1],
//                 blue: blue * rhs[2],
//                 alpha,
//             },
//             Color::RgbaLinear {
//                 red,
//                 green,
//                 blue,
//                 alpha,
//             } => Color::RgbaLinear {
//                 red: red * rhs[0],
//                 green: green * rhs[1],
//                 blue: blue * rhs[2],
//                 alpha,
//             },
//             Color::Hsla {
//                 hue,
//                 saturation,
//                 lightness,
//                 alpha,
//             } => Color::Hsla {
//                 hue: hue * rhs[0],
//                 saturation: saturation * rhs[1],
//                 lightness: lightness * rhs[2],
//                 alpha,
//             },
//         }
//     }
// }

// impl MulAssign<[f32; 3]> for Color {
//     fn mul_assign(&mut self, rhs: [f32; 3]) {
//         match self {
//             Color::Rgba {
//                 red, green, blue, ..
//             }
//             | Color::RgbaLinear {
//                 red, green, blue, ..
//             } => {
//                 *red *= rhs[0];
//                 *green *= rhs[1];
//                 *blue *= rhs[2];
//             }
//             Color::Hsla {
//                 hue,
//                 saturation,
//                 lightness,
//                 ..
//             } => {
//                 *hue *= rhs[0];
//                 *saturation *= rhs[1];
//                 *lightness *= rhs[2];
//             }
//         }
//     }
// }
