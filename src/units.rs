pub mod image_space {
    use std::ops;

    make_units! {
        PIX;
        ONE: Unitless;

        base {
            PX: Pixel, "px", Length;
        }

        derived {
            PX2: Pixel2 = (Pixel * Pixel), Area;
        }

        constants {
        }

        fmt = true;
    }

    macro_rules! define_conversion {
        ($Unit:ty, $ValueType:ty, $ResultType:tt) => {
            impl ops::Mul<$Unit> for $ValueType {
                type Output = $ResultType<$ValueType>;
                #[inline]
                fn mul(self, _:$Unit) -> Self::Output {
                   $ResultType::new(self)
                }
            }

            impl ops::Div<$Unit> for $ResultType<$ValueType> {
                type Output = $ValueType;
                #[inline]
                fn div(self, _:$Unit) -> Self::Output {
                    self.value_unsafe
                }
            }
        };
    }

    pub struct PX;

    define_conversion!(PX, i32, Pixel);
    define_conversion!(PX, i64, Pixel);
    define_conversion!(PX, isize, Pixel);

    define_conversion!(PX, f32, Pixel);
    define_conversion!(PX, f64, Pixel);
}

pub use self::image_space::{Pixel, PX};
pub type DistPx = Pixel<isize>;
pub type DistPxFrac = Pixel<f64>;


#[cfg(test)]
mod test {
    use super::{Pixel, PX};

    #[test]
    fn create_integer_values() {
        let x: Pixel<i64> = 42i64 * PX;
        assert_eq!(x / PX, 42);

        let y: Pixel<isize> = 42isize * PX;
        assert_eq!(y / PX, 42);
    }

    #[test]
    fn create_float_values() {
        let x = 42.0 * PX;
        assert_eq!((x / PX), 42.0);
    }
}
