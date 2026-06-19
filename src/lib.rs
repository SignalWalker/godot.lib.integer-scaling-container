use std::num::NonZero;

use godot::classes::class_macros::private::virtuals::ZipReader::Vector2i;

mod ext;
pub use ext::*;

mod integer_scaling_container;
pub use integer_scaling_container::*;

mod integer_scaling_subviewport_container;
pub use integer_scaling_subviewport_container::*;

pub const fn get_largest_multiple_within(base: NonZero<u32>, max: u32) -> u32 {
    max - max.rem_euclid(base.get())
}

pub fn get_largest_integer_scale(base: Vector2i, within: Vector2i) -> Option<u32> {
    let x = if base.x > 0 {
        unsafe { NonZero::new_unchecked(base.x as u32) }
    } else {
        return None;
    };
    let y = if base.y > 0 {
        unsafe { NonZero::new_unchecked(base.y as u32) }
    } else {
        return None;
    };
    let largest_y = get_largest_multiple_within(y, u32::try_from(within.y).ok()?) / y;
    let largest_x = get_largest_multiple_within(x, u32::try_from(within.x).ok()?) / x;
    Some(largest_x.min(largest_y))
}

#[cfg(test)]
mod test {
    macro_rules! assert_lrg {
        ({ $b_x:expr, $b_y:expr }, { $w_x:expr, $w_y:expr } => $expected:expr) => {{
            let __expected = $expected;
            let __base = ::godot::prelude::Vector2i { x: $b_x, y: $b_y };
            let __within = ::godot::prelude::Vector2i { x: $w_x, y: $w_y };
            ::std::assert_eq!(
                $crate::get_largest_integer_scale(__base, __within),
                __expected,
                "get_largest_multiple_within({}, {})",
                __base,
                __within
            )
        }};
    }

    #[test]
    fn test_largest_integer_scale() {
        assert_lrg!({0, 1}, {1, 1} => None);
        assert_lrg!({1, 0}, {1, 1} => None);
        assert_lrg!({1, 1}, {-1, 1} => None);
        assert_lrg!({1, 1}, {1, -1} => None);

        assert_lrg!({1, 1}, {2, 1} => Some(1));
        assert_lrg!({1, 1}, {1, 2} => Some(1));
        assert_lrg!({1, 1}, {2, 2} => Some(2));
    }
}
