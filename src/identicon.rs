use anyhow::anyhow;
use blake2_rfc::blake2b::blake2b;
use palette::{FromColor, FromComponent, Hsl, RgbHue, Srgb};
use svg::node::element;
use svg::Document;

const HALF_SVG: i32 = 32;
pub const FOREGROUND_COLOR: [u8; 4] = [238, 238, 238, 255];

pub fn make(into_id: &[u8]) -> String {
    svg_from_vec(&into_id, HALF_SVG)
        .to_string()
        .replace("\n", "")
}

/// Function to choose the coloring scheme based on value d.
/// Note that d is calculated as remainder of division by total sum of frequencies,
/// so it can not exceed the total sum of frequencies
fn choose_scheme(schemes: Vec<SchemeElement>, d: u32) -> anyhow::Result<SchemeElement> {
    let mut sum = 0;
    let mut found_scheme = None;
    for x in schemes.into_iter() {
        sum += x.freq as u32;
        if d < sum {
            found_scheme = Some(x);
            break;
        }
    }
    match found_scheme {
        Some(x) => Ok(x),
        None => Err(anyhow!("not accessible")),
    }
}

/// Struct to store default coloring schemes
struct SchemeElement {
    freq: u8,
    colors: [usize; 19],
}

/// Function to set default coloring schemes, taken as is from js code
fn default_schemes() -> Vec<SchemeElement> {
    vec![
        SchemeElement {
            // "target"
            freq: 1,
            colors: [
                0, 28, 0, 0, 28, 0, 0, 28, 0, 0, 28, 0, 0, 28, 0, 0, 28, 0, 1,
            ],
        },
        SchemeElement {
            // "cube",
            freq: 20,
            colors: [0, 1, 3, 2, 4, 3, 0, 1, 3, 2, 4, 3, 0, 1, 3, 2, 4, 3, 5],
        },
        SchemeElement {
            // "quazar",
            freq: 16,
            colors: [1, 2, 3, 1, 2, 4, 5, 5, 4, 1, 2, 3, 1, 2, 4, 5, 5, 4, 0],
        },
        SchemeElement {
            // "flower",
            freq: 32,
            colors: [0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 3],
        },
        SchemeElement {
            // "cyclic",
            freq: 32,
            colors: [0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6],
        },
        SchemeElement {
            // "vmirror",
            freq: 128,
            colors: [0, 1, 2, 3, 4, 5, 3, 4, 2, 0, 1, 6, 7, 8, 9, 7, 8, 6, 10],
        },
        SchemeElement {
            // "hmirror",
            freq: 128,
            colors: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 8, 6, 7, 5, 3, 4, 2, 11],
        },
    ]
}

/// Struct to store information about circle center position
/// For 19-circle icons circle positions are set as defaults
struct CirclePosition {
    x_center: i32,
    y_center: i32,
}

/// Helper function to transform RGBA [u8; 4] color needed for png into
/// hex string color needed for svg
fn rgba_to_hex(rgba_color: [u8; 4]) -> String {
    format!(
        "#{}",
        hex::encode(vec![rgba_color[0], rgba_color[1], rgba_color[2]])
    )
}

/// Function to set default positions of small circles in 19-circle icon
/// a is center_to_center distance
fn position_circle_set(a: i32) -> Vec<CirclePosition> {
    let b = ((a as f64) * 3f64.sqrt() / 2.0).round() as i32;
    vec![
        CirclePosition {
            x_center: 0,
            y_center: -2 * a,
        },
        CirclePosition {
            x_center: 0,
            y_center: -a,
        },
        CirclePosition {
            x_center: -b,
            y_center: -3 * a / 2,
        },
        CirclePosition {
            x_center: -2 * b,
            y_center: -a,
        },
        CirclePosition {
            x_center: -b,
            y_center: -a / 2,
        },
        CirclePosition {
            x_center: -2 * b,
            y_center: 0,
        },
        CirclePosition {
            x_center: -2 * b,
            y_center: a,
        },
        CirclePosition {
            x_center: -b,
            y_center: a / 2,
        },
        CirclePosition {
            x_center: -b,
            y_center: 3 * a / 2,
        },
        CirclePosition {
            x_center: 0,
            y_center: 2 * a,
        },
        CirclePosition {
            x_center: 0,
            y_center: a,
        },
        CirclePosition {
            x_center: b,
            y_center: 3 * a / 2,
        },
        CirclePosition {
            x_center: 2 * b,
            y_center: a,
        },
        CirclePosition {
            x_center: b,
            y_center: a / 2,
        },
        CirclePosition {
            x_center: 2 * b,
            y_center: 0,
        },
        CirclePosition {
            x_center: 2 * b,
            y_center: -a,
        },
        CirclePosition {
            x_center: b,
            y_center: -a / 2,
        },
        CirclePosition {
            x_center: b,
            y_center: -3 * a / 2,
        },
        CirclePosition {
            x_center: 0,
            y_center: 0,
        },
    ]
}

pub fn svg_from_vec(into_id: &[u8], halfsize: i32) -> Document {
    let mut document = Document::new().set(
        "viewBox",
        (-halfsize, -halfsize, 2 * halfsize, 2 * halfsize),
    );
    let colors = get_colors_from_vec(&into_id);
    let data = calculate_svg_data(halfsize, colors);
    for x in data.into_iter() {
        document = document.add(x);
    }
    document
}

/// Function to calculate svg file contents (using element::Circle from svg crate)
pub fn calculate_svg_data(big_radius: i32, colors: Vec<[u8; 4]>) -> Vec<element::Circle> {
    let mut out: Vec<element::Circle> = Vec::with_capacity(20);
    out.push(
        element::Circle::new()
            .set("cx", 0)
            .set("cy", 0)
            .set("r", big_radius)
            .set("fill", rgba_to_hex(FOREGROUND_COLOR))
            .set("stroke", "none"),
    );
    let small_radius = big_radius / 32 * 5;
    let center_to_center = big_radius / 8 * 3;
    let positions = position_circle_set(center_to_center);
    for (i, position) in positions.iter().enumerate() {
        out.push(
            element::Circle::new()
                .set("cx", position.x_center)
                .set("cy", position.y_center)
                .set("r", small_radius)
                .set("fill", rgba_to_hex(colors[i]))
                .set("stroke", "none"),
        );
    }
    out
}

/// Function to get colors from u8 vector
pub fn get_colors_from_vec(into_id: &[u8]) -> Vec<[u8; 4]> {
    let into_zero = [0u8; 32].to_vec();
    let zero = blake2b(64, &[], &into_zero).as_bytes().to_vec();

    let id_prep = blake2b(64, &[], &into_id).as_bytes().to_vec();

    let mut id: Vec<u8> = Vec::new();
    for (i, x) in id_prep.iter().enumerate() {
        let new = x.wrapping_sub(zero[i]);
        id.push(new);
    }

    // Since `id[29]` is u8, `sat` could range from 30 to 109, i.e. it always fits into u8.
    // Transformation of id[29] into u16 is to avoid overflow in multiplication (wrapping could be used, but is more bulky).
    // TODO For color calculation `sat` is used as saturation in percents
    // (this is taken as is from js code).
    // However, this way `sat_component` could have values above 1.00.
    // Palette crate does not check at this moment that `sat_component` is not overflowing 1.00, and produces
    // some kind of resulting color.
    // Need to find out what should have happened if the sat values are above 100.
    let sat = (((id[29] as u16 * 70 / 256 + 26) % 80) + 30) as u8;
    let sat_component: f64 = (sat as f64) / 100.0;

    // calculating palette: set of 32 RGBA colors to be used is drawing
    // only id vector is used for this calculation
    let mut my_palette: Vec<[u8; 4]> = Vec::new();
    for (i, x) in id.iter().enumerate() {
        let b = x.wrapping_add((i as u8 % 28).wrapping_mul(58));
        let new = match b {
            0 => [4, 4, 4, 255],
            255 => FOREGROUND_COLOR, // transparent
            _ => {
                // HSL color hue in degrees
                // calculated as integer, same as in js code
                // transformation to u16 is done to avoid overflow
                let h = (b as u16 % 64 * 360) / 64;
                // recalculated into RgbHue, to be used as HSL hue component
                let h_component = RgbHue::from_degrees(h as f64);

                // HSL lightness in percents
                let l: u8 = match b / 64 {
                    0 => 53,
                    1 => 15,
                    2 => 35,
                    _ => 75,
                };
                // recalculated in HSL lightness component (component range is 0.00 to 1.00)
                let l_component: f64 = (l as f64) / 100.0;

                // defining HSL color
                let color_hsl = Hsl::new(h_component, sat_component, l_component);

                // transforming HSL color into RGB color, possibly lossy, TODO check if too lossy
                let color_srgb = Srgb::from_color(color_hsl);

                // getting red, green, blue components, transforming them in 0..255 range of u8
                let red = u8::from_component(color_srgb.red);
                let green = u8::from_component(color_srgb.green);
                let blue = u8::from_component(color_srgb.blue);

                // finalize color to add to palette, not transparent
                [red, green, blue, 255]
            }
        };
        my_palette.push(new);
    }

    // loading default coloring schemes
    let schemes = default_schemes();

    // `total` is the sum of frequencies for all scheme elements in coloring schemes,
    // in current setting is always 357
    let mut total = 0;
    for x in schemes.iter() {
        total += x.freq as u32;
    }

    // `d` is used to determine the coloring scheme to be used.
    // Transformation into u32 is used to avoid overflow.
    let d = (id[30] as u32 + (id[31] as u32) * 256) % total;

    // determining the coloring scheme to be used
    let my_scheme = choose_scheme(schemes, d).expect("should always work: d is calculated as remainder of division by total sum of frequencies, so it can not exceed the total sum of frequencies");

    // calculating rotation for the coloring scheme
    let rot = (id[28] % 6) * 3;

    // picking colors from palette using coloring scheme with rotation applied
    let mut my_colors: Vec<[u8; 4]> = Vec::with_capacity(19);
    for i in 0..19 {
        let num_color = {
            if i < 18 {
                (i + rot) % 18
            } else {
                18
            }
        } as usize;
        let num_palette = my_scheme.colors[num_color];
        let color = my_palette[num_palette];
        my_colors.push(color);
    }

    my_colors
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use crate::identicon::make;

    #[test]
    fn test_make() {
        let actual = make(&[1]);
        assert_eq!("<svg viewBox=\"-32 -32 64 64\" xmlns=\"http://www.w3.org/2000/svg\"><circle cx=\"0\" cy=\"0\" fill=\"#eeeeee\" r=\"32\" stroke=\"none\"/><circle cx=\"0\" cy=\"-24\" fill=\"#d4aabf\" r=\"5\" stroke=\"none\"/><circle cx=\"0\" cy=\"-12\" fill=\"#d4caaa\" r=\"5\" stroke=\"none\"/><circle cx=\"-10\" cy=\"-18\" fill=\"#3d3c77\" r=\"5\" stroke=\"none\"/><circle cx=\"-20\" cy=\"-12\" fill=\"#77623c\" r=\"5\" stroke=\"none\"/><circle cx=\"-10\" cy=\"-6\" fill=\"#af7560\" r=\"5\" stroke=\"none\"/><circle cx=\"-20\" cy=\"0\" fill=\"#3d3c77\" r=\"5\" stroke=\"none\"/><circle cx=\"-20\" cy=\"12\" fill=\"#d4aabf\" r=\"5\" stroke=\"none\"/><circle cx=\"-10\" cy=\"6\" fill=\"#d4caaa\" r=\"5\" stroke=\"none\"/><circle cx=\"-10\" cy=\"18\" fill=\"#3d3c77\" r=\"5\" stroke=\"none\"/><circle cx=\"0\" cy=\"24\" fill=\"#77623c\" r=\"5\" stroke=\"none\"/><circle cx=\"0\" cy=\"12\" fill=\"#af7560\" r=\"5\" stroke=\"none\"/><circle cx=\"10\" cy=\"18\" fill=\"#3d3c77\" r=\"5\" stroke=\"none\"/><circle cx=\"20\" cy=\"12\" fill=\"#d4aabf\" r=\"5\" stroke=\"none\"/><circle cx=\"10\" cy=\"6\" fill=\"#d4caaa\" r=\"5\" stroke=\"none\"/><circle cx=\"20\" cy=\"0\" fill=\"#3d3c77\" r=\"5\" stroke=\"none\"/><circle cx=\"20\" cy=\"-12\" fill=\"#77623c\" r=\"5\" stroke=\"none\"/><circle cx=\"10\" cy=\"-6\" fill=\"#af7560\" r=\"5\" stroke=\"none\"/><circle cx=\"10\" cy=\"-18\" fill=\"#3d3c77\" r=\"5\" stroke=\"none\"/><circle cx=\"0\" cy=\"0\" fill=\"#5e3c77\" r=\"5\" stroke=\"none\"/></svg>", actual);
    }
}
