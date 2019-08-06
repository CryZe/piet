//! A simple representation of color

use std::fmt::{Debug, Formatter};

/// A datatype representing color.
///
/// Currently this is only a 32 bit RGBA value, but it will likely
/// extend to some form of wide-gamut colorspace, and in the meantime
/// is useful for giving programs proper type.
#[derive(Clone)]
pub enum Color {
    Rgba32(u32),
}

impl Debug for Color {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Color(#{:08x})", self.as_rgba32())
    }
}

impl Color {
    /// Create a color from a 32-bit rgba value (alpha as least significant byte).
    pub const fn rgba32(rgba: u32) -> Color {
        Color::Rgba32(rgba)
    }

    /// Create a color from a 24-bit rgb value (red most significant, blue least).
    pub const fn rgb24(rgb: u32) -> Color {
        Color::rgba32((rgb << 8) | 0xff)
    }

    /// Create a color from four floating point values, each in the range 0.0 to 1.0.
    ///
    /// The interpretation is the same as rgba32, and no greater precision is
    /// (currently) assumed.
    pub fn rgba<F: Into<f64>>(r: F, g: F, b: F, a: F) -> Color {
        let r = (r.into().max(0.0).min(1.0) * 255.0).round() as u32;
        let g = (g.into().max(0.0).min(1.0) * 255.0).round() as u32;
        let b = (b.into().max(0.0).min(1.0) * 255.0).round() as u32;
        let a = (a.into().max(0.0).min(1.0) * 255.0).round() as u32;
        Color::rgba32((r << 24) | (g << 16) | (b << 8) | a)
    }

    /// Create a color from three floating point values, each in the range 0.0 to 1.0.
    ///
    /// The interpretation is the same as rgb24, and no greater precision is
    /// (currently) assumed.
    pub fn rgb<F: Into<f64>>(r: F, g: F, b: F) -> Color {
        let r = (r.into().max(0.0).min(1.0) * 255.0).round() as u32;
        let g = (g.into().max(0.0).min(1.0) * 255.0).round() as u32;
        let b = (b.into().max(0.0).min(1.0) * 255.0).round() as u32;
        Color::rgba32((r << 24) | (g << 16) | (b << 8) | 0xff)
    }

    /// Create a color from a CIEL\*a\*b\* polar (also known as CIE HCL)
    /// specification.
    ///
    /// The `h` parameter is an angle in degrees, with 0 roughly magenta, 90
    /// roughly yellow, 180 roughly cyan, and 270 roughly blue. The `l`
    /// parameter is perceptual luminance, with 0 black and 100 white.
    /// The `c` parameter is a chrominance concentration, with 0 grayscale
    /// and a nominal maximum of 127 (in the future, higher values might
    /// be useful, for high gamut contexts).
    ///
    /// Currently this is just converted into sRGB, but in the future as we
    /// support high-gamut colorspaces, it can be used to specify more colors
    /// or existing colors with a higher accuracy.
    ///
    /// Currently out-of-gamut values are clipped to the nearest sRGB color,
    /// which is perhaps not ideal (the clipping might change the hue). See
    /// https://github.com/d3/d3-color/issues/33 for discussion.
    #[allow(non_snake_case)]
    pub fn hlc<F: Into<f64>>(h: F, l: F, c: F) -> Color {
        // The reverse transformation from Lab to XYZ, see
        // https://en.wikipedia.org/wiki/CIELAB_color_space
        fn f_inv(t: f64) -> f64 {
            let d = 6. / 29.;
            if t > d {
                t.powi(3)
            } else {
                3. * d * d * (t - 4. / 29.)
            }
        }
        let th = h.into() * (std::f64::consts::PI / 180.);
        let c = c.into();
        let a = c * th.cos();
        let b = c * th.sin();
        let L = l.into();
        let ll = (L + 16.) * (1. / 116.);
        // Produce raw XYZ values
        let X = f_inv(ll + a * (1. / 500.));
        let Y = f_inv(ll);
        let Z = f_inv(ll - b * (1. / 200.));
        // This matrix is the concatenation of three sources.
        // First, the white point is taken to be ICC standard D50, so
        // the diagonal matrix of [0.9642, 1, 0.8249]. Note that there
        // is some controversy around this value. However, it matches
        // the other matrices, thus minimizing chroma error.
        //
        // Second, an adaption matrix from D50 to D65. This is the
        // inverse of the recommended D50 to D65 adaptation matrix
        // from the W3C sRGB spec:
        // https://www.w3.org/Graphics/Color/srgb
        //
        // Finally, the conversion from XYZ to linear sRGB values,
        // also taken from the W3C sRGB spec.
        let r_lin = 3.02172918 * X - 1.61692294 * Y - 0.40480625 * Z;
        let g_lin = -0.94339358 * X + 1.91584267 * Y + 0.02755094 * Z;
        let b_lin = 0.06945666 * X - 0.22903204 * Y + 1.15957526 * Z;
        fn gamma(u: f64) -> f64 {
            if u <= 0.0031308 {
                12.92 * u
            } else {
                1.055 * u.powf(1. / 2.4) - 0.055
            }
        }
        Color::rgb(gamma(r_lin), gamma(g_lin), gamma(b_lin))
    }

    /// Create a color from a CIEL\*a\*b\* polar specification and alpha.
    ///
    /// The `a` value represents alpha in the range 0.0 to 1.0.
    pub fn hlca<F: Into<f64>>(h: F, l: F, c: F, a: impl Into<f64>) -> Color {
        Color::hlc(h, c, l).with_alpha(a)
    }

    /// Change just the alpha value of a color.
    ///
    /// The `a` value represents alpha in the range 0.0 to 1.0.
    pub fn with_alpha(self, a: impl Into<f64>) -> Color {
        let a = (a.into().max(0.0).min(1.0) * 255.0).round() as u32;
        Color::rgba32((self.as_rgba32() & !0xff) | a)
    }

    /// Convert a color value to a 32-bit rgba value.
    pub fn as_rgba32(&self) -> u32 {
        match *self {
            Color::Rgba32(rgba) => rgba,
        }
    }

    /// Opaque white.
    pub const WHITE: Color = Color::rgba32(0xff_ff_ff_ff);

    /// Opaque black.
    pub const BLACK: Color = Color::rgba32(0x00_00_00_ff);
}
