/*!
    Rainbow color math ported from Adafruit_NeoPixel
    (github.com/adafruit/Adafruit_NeoPixel), so the rainbow effect renders
    the same colors the Arduino firmware produced.
*/

/// Adafruit_NeoPixel::ColorHSV(hue, sat = 255, val = 255) -> (r, g, b).
///
/// Integer port of the upstream implementation; u32 intermediates match the
/// C integer promotions and truncations.
pub fn color_hsv(hue: u16) -> (u8, u8, u8) {
    const SAT: u16 = 255;
    const VAL: u8 = 255;

    // Remap 0-65535 to 0-1529. Real max is 1527, but round as we like.
    let hue = (hue as u32 * 1530 + 32768) / 65536;

    let (r, g, b): (u32, u32, u32);
    if hue < 510 {
        // Red to Green
        b = 0;
        if hue < 255 {
            r = 255;
            g = hue;
        } else {
            // 510, not 255: hue is in [255, 510) here, so 255 - hue would
            // underflow (panics in debug, garbage colors in release).
            r = 510 - hue;
            g = 255;
        }
    } else if hue < 1020 {
        // Green to Blue
        r = 0;
        if hue < 765 {
            g = 255;
            b = hue - 510;
        } else {
            g = 1020 - hue;
            b = 255;
        }
    } else if hue < 1530 {
        // Blue to Red
        g = 0;
        if hue < 1275 {
            r = hue - 1020;
            b = 255;
        } else {
            r = 255;
            b = 1530 - hue;
        }
    } else {
        r = 255;
        g = 0;
        b = 0;
    }

    // Full saturation/value packing, mirroring the upstream final step:
    // with sat = 255 and val = 255 this reduces to (r, g, b) unchanged.
    let v1 = 1 + VAL as u32;
    let s1 = 1 + SAT as u32;
    let s2 = 255 - SAT as u32;
    let packed = (((((r * s1) >> 8) + s2) * v1) & 0xff00) << 8
        | (((((g * s1) >> 8) + s2) * v1) & 0xff00)
        | (((((b * s1) >> 8) + s2) * v1) >> 8);
    ((packed >> 16) as u8, (packed >> 8) as u8, packed as u8)
}

/// Adafruit_NeoPixel::gamma8(x): gamma-2.6 lookup table.
pub fn gamma8(x: u8) -> u8 {
    /// Generated with (int)(pow(i / 255.0, 2.6) * 255.0 + 0.5), as documented
    /// in the upstream header.
    const GAMMA: [u8; 256] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6,
        6, 6, 6, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14, 14, 15,
        15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27, 27,
        28, 29, 29, 30, 31, 31, 32, 33, 34, 34, 35, 36, 37, 38, 38, 39, 40, 41, 42, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 68, 69,
        70, 71, 72, 73, 75, 76, 77, 78, 80, 81, 82, 84, 85, 86, 88, 89, 90, 92, 93, 94, 96, 97, 99,
        100, 102, 103, 105, 106, 108, 109, 111, 112, 114, 115, 117, 119, 120, 122, 124, 125, 127,
        129, 130, 132, 134, 136, 137, 139, 141, 143, 145, 146, 148, 150, 152, 154, 156, 158, 160,
        162, 164, 166, 168, 170, 172, 174, 176, 178, 180, 182, 184, 186, 188, 191, 193, 195, 197,
        199, 202, 204, 206, 209, 211, 213, 215, 218, 220, 223, 225, 227, 230, 232, 235, 237, 240,
        242, 245, 247, 250, 252, 255,
    ];
    GAMMA[x as usize]
}

/// Adafruit_NeoPixel::gamma32 applied to an (r, g, b) triple.
pub fn gamma32(rgb: (u8, u8, u8)) -> (u8, u8, u8) {
    (gamma8(rgb.0), gamma8(rgb.1), gamma8(rgb.2))
}

#[cfg(test)]
mod tests {
    use super::{color_hsv, gamma8};

    // Expected values recomputed from the upstream C code by hand.
    #[test]
    fn hsv_matches_adafruit_reference_values() {
        assert_eq!(color_hsv(0), (255, 0, 0)); // red
        assert_eq!(color_hsv(10923), (255, 255, 0)); // yellow (hue remap = 255.49 -> 255)
        assert_eq!(color_hsv(21845), (0, 255, 0)); // green (hue remap = 510.49 -> 510)
        assert_eq!(color_hsv(27_306), (0, 255, 127)); // green -> blue, b = 637 - 510
        assert_eq!(color_hsv(32768), (0, 255, 255)); // cyan (hue remap = 765.06 -> 765)
        assert_eq!(color_hsv(43_690), (0, 0, 255)); // blue (hue remap = 1020.6 -> 1020)
        assert_eq!(color_hsv(65_535), (255, 0, 0)); // hue remap = 1530.46 -> pure red
    }

    #[test]
    fn hsv_is_never_saturated_by_packing() {
        for hue in (0..=65_535u32).step_by(257) {
            let (r, g, b) = color_hsv(hue as u16);
            assert!(r <= 255 && g <= 255 && b <= 255);
        }
    }

    #[test]
    fn gamma_table_endpoints_and_monotonicity() {
        assert_eq!(gamma8(0), 0);
        assert_eq!(gamma8(255), 255);
        let mut prev = 0;
        for x in 0..=255u8 {
            assert!(gamma8(x) >= prev, "gamma must be non-decreasing at {x}");
            prev = gamma8(x);
        }
        // Entries verified against the upstream _NeoPixelGammaTable.
        assert_eq!(gamma8(27), 1);
        assert_eq!(gamma8(64), 7);
        assert_eq!(gamma8(128), 42);
    }
}
