use std::path::Path;
use std::io::{Error, Result};
use std::ops;

use memmap::{Mmap, Protection};
use num::{FromPrimitive, Num};
use units::{DistPx, PX};

pub trait Pixel: Num + Sized + Copy + FromPrimitive {
    #[cfg(test)]
    fn bytes<'a>(&'a self) -> &'a [u8];
}

macro_rules! impl_pixel {
    ($($t:ty),*) => ($(
        impl Pixel for $t {
            #[cfg(test)]
            fn bytes<'a>(&'a self) -> &'a[u8] {
                use std::mem;
                use std::slice;

                // is there a safe way to do this generically?
                let p : *const $t = self;
                unsafe {
                    slice::from_raw_parts(p as *const u8, mem::size_of::<$t>())
                }
            }
        }
    )*)
}

impl_pixel!(i16, i32, f32);

pub trait Image<PixelType: Pixel>
    : ops::Index<(DistPx, DistPx), Output = PixelType> {
    /// Fetch the dimensions of the image
    fn dimensions(&self) -> (DistPx, DistPx);

    /// Fetches an immutable slice containing all the pixels in the image in
    /// scan-major order. There is no padding between scan lines.
    fn pixels<'a>(&'a self) -> &'a [PixelType];
}

pub trait MutableImage<PixelType: Pixel>
    : Image<PixelType> + ops::IndexMut<(DistPx, DistPx)> {
    /// Fetches a mutable slice containing all the pixels in the image in
    /// scan-major order. There is no padding between scan lines.
    fn pixels_mut<'a>(&'a mut self) -> &'a mut [PixelType];

    /// Fills the image with pixels with a given value
    fn fill(&mut self, v: PixelType) {
        for p in self.pixels_mut().iter_mut() {
            *p = v;
        }
    }
}

// ----------------------------------------------------------------------------
// Owned image
// ----------------------------------------------------------------------------

/// Defines a potentially-mutable image that owns its own pixels.
pub struct OwnedImage<PixelType: Pixel> {
    width: DistPx,
    height: DistPx,
    pixels: Vec<PixelType>,
}

impl<PixelType: Pixel> OwnedImage<PixelType> {
    pub fn new(width: DistPx, height: DistPx) -> OwnedImage<PixelType> {
        let size = ((width / PX) * (height / PX)) as usize;
        OwnedImage {
            width: width,
            height: height,
            pixels: vec![PixelType::zero(); size],
        }
    }
}

impl<PixelType: Pixel> ops::Index<(DistPx, DistPx)> for OwnedImage<PixelType> {
    type Output = PixelType;

    fn index(&self, coords: (DistPx, DistPx)) -> &PixelType {
        let (x, y) = coords;
        let offset = ((y / PX * (self.width / PX)) + (x / PX)) as usize;
        &self.pixels[offset]
    }
}

impl<PixelType: Pixel> ops::IndexMut<(DistPx, DistPx)>
    for OwnedImage<PixelType> {
    fn index_mut(&mut self, coords: (DistPx, DistPx)) -> &mut PixelType {
        let (x, y) = coords;
        let offset = ((y / PX * (self.width / PX)) + (x / PX)) as usize;
        &mut self.pixels[offset]
    }
}


impl<PixelType: Pixel> Image<PixelType> for OwnedImage<PixelType> {
    fn dimensions(&self) -> (DistPx, DistPx) {
        (self.width, self.height)
    }

    fn pixels<'a>(&'a self) -> &'a [PixelType] {
        self.pixels.as_slice()
    }
}

impl<PixelType: Pixel> MutableImage<PixelType> for OwnedImage<PixelType> {
    fn pixels_mut<'a>(&'a mut self) -> &'a mut [PixelType] {
        self.pixels.as_mut_slice()
    }
}

#[cfg(test)]
mod test_owned_image {
    use units::DistPx;
    use super::*;

    #[test]
    fn create() {
        let w = DistPx::new(256);
        let h = DistPx::new(128);
        let i = OwnedImage::<i32>::new(w, h);

        assert_eq!(i.width, w);
        assert_eq!(i.height, h);
        assert_eq!(i.pixels.len(), ((w / PX) * (h / PX)) as usize);
        assert!(i.pixels.iter().all(|x| *x == 0));
    }

    #[test]
    fn indexing() {
        let w = 256isize;
        let h = 128isize;
        let mut mut_img = OwnedImage::<i32>::new(w * PX, h * PX);

        for y in 0..h {
            for x in 0..w {
                mut_img[(x * PX, y * PX)] = ((1000 * y) + x) as i32;
            }
        }

        let img = &mut_img;
        for y in 0..h {
            for x in 0..w {
                assert_eq!(img[(x * PX, y * PX)], ((1000 * y) + x) as i32);
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Memory-mapped image
// ----------------------------------------------------------------------------

/// An immutable image loaded from file into a memory-mapped buffer.
pub struct MemoryMappedImage<'a, PixelType: Pixel + 'a> {
    width: DistPx,
    height: DistPx,
    mem: Mmap,
    pixels: &'a [PixelType],
}

impl<'a, PixelType: Pixel + 'a> MemoryMappedImage<'a, PixelType> {
    /// Map an image file into memory and encapsulate it inside a
    /// MemoryMappedImage
    pub fn map_file(path: &Path,
                    width: DistPx,
                    height: DistPx)
                    -> Result<MemoryMappedImage<PixelType>> {
        use std::mem;
        use std::slice;
        use std::io::ErrorKind;

        debug!("Mapping file: {:?}", path);
        let map = Mmap::open_path(path, Protection::Read)?;

        let expected_size = ((width / PX) * (height / PX)) as usize *
                            mem::size_of::<PixelType>();
        if map.len() != expected_size {
            return Err(Error::new(ErrorKind::Other, "Unexpected size"));
        }

        let pixels = unsafe {
            slice::from_raw_parts(map.ptr() as *const PixelType,
                                  map.len() / mem::size_of::<PixelType>())
        };

        let result = MemoryMappedImage {
            width: width,
            height: height,
            mem: map,
            pixels: pixels,
        };

        Ok(result)
    }
}

impl<'a, PixelType: Pixel> ops::Index<(DistPx, DistPx)>
    for
    MemoryMappedImage<'a, PixelType> {
    type Output = PixelType;

    fn index(&self, coords: (DistPx, DistPx)) -> &PixelType {
        let (x, y) = coords;
        let offset = ((y / PX * (self.width / PX)) + (x / PX)) as usize;
        &self.pixels[offset]
    }
}

impl<'a, PixelType: Pixel> Image<PixelType>
    for MemoryMappedImage<'a, PixelType> {
    fn dimensions(&self) -> (DistPx, DistPx) {
        (self.width, self.height)
    }

    fn pixels<'b>(&'b self) -> &'b [PixelType] {
        self.pixels
    }
}

#[cfg(test)]
mod test_memory_mapped_image {
    use super::*;

    use tempfile::NamedTempFile;
    use units::{DistPx, PX};

    fn make_test_image<PixelType: Pixel>(width: DistPx,
                                         height: DistPx)
                                         -> NamedTempFile {
        use std::io::Write;
        use std::fs::File;

        let tmp = NamedTempFile::new().unwrap();

        debug!("Creating tmp image file at {:?}", tmp.path());
        let mut f = File::create(tmp.path()).unwrap();

        for y in 0..height / PX {
            for x in 0..width / PX {
                let px = PixelType::from_isize((1000 * y) + x).unwrap();
                f.write_all(px.bytes()).unwrap();
            }
        }

        tmp
    }

    #[test]
    fn mapping_an_i32_file() {
        let width = 256isize * PX;
        let height = 128isize * PX;

        let tmp = make_test_image::<i32>(width, height);
        let img = MemoryMappedImage::<i32>::map_file(tmp.path(), width, height)
            .unwrap();

        assert_eq!(img.width, width);
        assert_eq!(img.height, height);
        assert_eq!(img.pixels.len(), ((width / PX) * (height / PX)) as usize);

        for y in 0..height / PX {
            for x in 0..width / PX {
                let px = img[(x * PX, y * PX)];
                assert_eq!(((y * 1000) + x) as i32, px);
            }
        }
    }

    #[test]
    fn mapping_an_f32_file() {
        let width = 256isize * PX;
        let height = 128isize * PX;

        let tmp = make_test_image::<f32>(width, height);
        let img = MemoryMappedImage::<f32>::map_file(tmp.path(), width, height)
            .unwrap();
        for y in 0..height / PX {
            for x in 0..width / PX {
                let px = img[(x * PX, y * PX)];
                assert_eq!(((y * 1000) + x) as f32, px);
            }
        }
    }
}
