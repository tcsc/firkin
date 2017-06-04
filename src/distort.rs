use units::{PX, DistPx, DistPxFrac};
use image::Image;
use num;

/// Maps a corrected pixel position in the destination image to an uncorrected
/// source pixel location, with sub-pixel accuracy.
fn map_dst_pixel(u: DistPx, y: DistPx) -> (DistPxFrac, DistPxFrac) {
    (0.0 * PX, 0.0 * PX)
}

/// Samples a sub-pixel point on the source image by synthesizing a new pixel
/// via bilinear filtering.
fn sample_image<ImageType>(i: &ImageType, u: DistPxFrac, v: DistPxFrac) -> i16
    where ImageType: Image<i16>
{
    let one = DistPx::new(1);
    let max_value = i16::max_value() as f64;

    // +-------+-------+
    // |A      |B      |
    // |   *   |       |
    // | (u,v) |       |
    // +-------+-------+
    // |C      |D      |
    // |       |       |
    // |       |       |
    // +-------+-------+

    // Remove the units from the coordinates u,v: they'll just make the
    // maths more murky
    let (u0, v0) = (u / PX, v / PX);

    // work out the top-left (i.e. "A") pixel to sample
    let (x0, y0) = (u0.floor(), v0.floor());

    // work out the contributions of the pixels in front and behind the
    // original u,v point
    let (col_1_contrib, row_1_contrib) = (u0 - x0, v0 - y0);
    let (col_0_contrib, row_0_contrib) = (1.0 - col_1_contrib,
                                          1.0 - row_1_contrib);

    // convert x0 & y0 back into integral pixel distances so that we can
    // actually use them to index the image pixels
    let (x, y) = (x0 as isize * PX, y0 as isize * PX);

    // sample the pixels that will contribute to the outpit
    let a = pixel_or_black(i, x, y);
    let b = pixel_or_black(i, x + one, y);
    let c = pixel_or_black(i, x, y + one);
    let d = pixel_or_black(i, x + one, y + one);

    // combine the pixels together to synthesize a new pixel value
    let new_pixel =
        ((a * col_0_contrib + b * col_1_contrib) * row_0_contrib) +
        ((c * col_0_contrib + d * col_1_contrib) * row_1_contrib);

    num::clamp(new_pixel, 0.0, max_value).round() as i16
}

#[inline]
fn pixel_or_black<ImageType>(i: &ImageType, x: DistPx, y: DistPx) -> f64
    where ImageType: Image<i16>
{
    let zero = DistPx::new(0isize);
    let (width, height) = i.dimensions();
    if (x < zero) || (y < zero) || (x >= width) || (y >= height) {
        0.0
    } else {
        i[(x, y)] as f64
    }
}

#[cfg(test)]
mod test_sampling {
    use super::sample_image;
    use image::{self, OwnedImage, MutableImage};
    use units::{self, PX, DistPx};

    #[test]
    fn identity_sample() {
        //    0     1     2
        // +-----+-----+-----+ Asserts that when the sample point (*) is the
        // |     |     |     | centre of a pixel, the value returned by the is
        // +-----+-----+-----+ approximately equal to the original pixel value.
        // |     |/////|     |
        // +-----+-----+-----+
        // |     |     |     |
        // +-----+-----+-----+

        let mut img = OwnedImage::<i16>::new(3isize * PX, 3isize * PX);
        img.fill(0);
        img[(1isize * PX, 1isize * PX)] = 2048;
        let rval = sample_image(&img, 1.0 * PX, 1.0 * PX);
        assert_eq!(rval, 2048)
    }

    #[test]
    fn bounds_check() {
        //    0     1     2
        // +-----+-----+-----+ Asserts that only the pixels touched by the
        // |     |     |     | sample window contribute to the sampled value.
        // +-----+-----+-----+ All other pixels are at full intensity, and
        // |     |  *--|--+  | should impact the result if included
        // +-----+--|--+--|--+
        // |     |  +--|--+  |
        // +-----+-----+-----+

        let mut img = OwnedImage::<i16>::new(3isize * PX, 3isize * PX);
        img.fill(i16::max_value());
        img[(1isize * PX, 1isize * PX)] = 48;
        img[(2isize * PX, 1isize * PX)] = 48;
        img[(1isize * PX, 2isize * PX)] = 48;
        img[(2isize * PX, 2isize * PX)] = 48;

        let rval = sample_image(&img, 1.5 * PX, 1.5 * PX);
        assert_eq!(rval, 48)
    }

    #[test]
    fn x_axis_averaging() {
        //    0     1     2
        // +-----+-----+-----+  Asserts that he horizontal (i.e. x-axis)
        // | BBB | WWW | BBB |  averaging works as expected by constructing a
        // +-----+-----+-----+  test image that has a black column and a white
        // | BBB | WWW | BBB |  column, and then resampling at points along the
        // +-----+-----+-----+  x axis and ensuring the resulting values change
        // | BBB | WWW | BBB |  as expected.
        // +-----+-----+-----+

        let one = DistPx::new(1);
        let mut img = OwnedImage::<i16>::new(3isize * PX, 3isize * PX);
        img.fill(0);
        for y in 0..3 {
            img[(one, (y as isize) * PX)] = 1024;
        }

        let test_cases = vec![(0.00f64 * PX, 0),
                              (0.25f64 * PX, 256),
                              (0.50f64 * PX, 512),
                              (0.75f64 * PX, 768),
                              (1.00f64 * PX, 1024)];

        for (offset, expected) in test_cases {
            let rval = sample_image(&img, offset, 1.0f64 * PX);
            assert_eq!(rval, expected);
        }
    }

    #[test]
    fn y_axis_averaging() {
        //    0     1     2
        // +-----+-----+-----+  Asserts that he vertical (i.e. y-axis)
        // | BBB | BBB | BBB |  averaging works as expected by constructing a
        // +-----+-----+-----+  test image that has a black row and a white row
        // | WWW | WWW | WWW |  and then resampling at points along the y-axis
        // +-----+-----+-----+  and ensuring the resulting values change as
        // | BBB | BBB | BBB |  expected.
        // +-----+-----+-----+

        let one = DistPx::new(1);
        let mut img = OwnedImage::<i16>::new(3isize * PX, 3isize * PX);
        img.fill(0);
        for x in 0..3 {
            img[((x as isize) * PX, one)] = 1024;
        }

        let test_cases = vec![(0.00f64 * PX, 0),
                              (0.25f64 * PX, 256),
                              (0.50f64 * PX, 512),
                              (0.75f64 * PX, 768),
                              (1.00f64 * PX, 1024)];

        for (offset, expected) in test_cases {
            let rval = sample_image(&img, 1.0f64 * PX, offset);
            assert_eq!(rval, expected);
        }
    }
}