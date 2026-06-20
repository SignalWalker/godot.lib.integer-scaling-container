use std::num::NonZero;

use godot::prelude::Vector2i;

mod ext;
pub use ext::*;

mod integer_scaling_container;
pub use integer_scaling_container::*;

mod integer_scaling_subviewport_container;
pub use integer_scaling_subviewport_container::*;

/// Get the largest integer multiple of `base` that is still smaller than `max`.
///
/// Note that 0 is a valid result.
pub const fn get_largest_multiple_within(base: NonZero<u32>, max: u32) -> u32 {
    max - max.rem_euclid(base.get())
}

/// Get the largest integer by which `base` can be multiplied and still have both components be
/// smaller than the components of `within`.
///
/// Returns `None` if either component of `base` is 0, or if either component of `within` is negative.
///
/// Note that if either component of `within` is 0, this will return `Some(0)`.
pub fn get_largest_integer_scale(base: Vector2i, within: Vector2i) -> Option<u32> {
    const fn to_nonzero_or_none(v: i32) -> Option<NonZero<u32>> {
        if v > 0 {
            Some(unsafe { NonZero::new_unchecked(v as u32) })
        } else {
            None
        }
    }
    let x = to_nonzero_or_none(base.x)?;
    let y = to_nonzero_or_none(base.y)?;
    let w_x = u32::try_from(within.x).ok()?;
    let w_y = u32::try_from(within.y).ok()?;
    let largest_y = get_largest_multiple_within(y, w_y) / y;
    let largest_x = get_largest_multiple_within(x, w_x) / x;
    Some(largest_x.min(largest_y))
}

/// Resize the given window to be the largest integer multiple of the given minimum size that
/// fits within its current screen.
pub fn fit_window(win: &mut godot::prelude::Gd<godot::classes::Window>, min_size: Vector2i) {
    if win.is_embedded() {
        // TODO :: why is this bad (and is this fixed by that one change in 4.7?)
        tracing::error!(
            window = %win,
            "called fit_window on an embedded window",
        );
        return;
    }
    if <godot::classes::Engine as godot::obj::Singleton>::singleton().is_embedded_in_editor() {
        // TODO :: why
        return;
    }

    win.set_min_size(min_size);

    let dsp = <godot::classes::DisplayServer as godot::obj::Singleton>::singleton();

    let id = win.get_window_id();
    let brd_size = dsp
        .window_get_size_with_decorations_ex()
        .window_id(id)
        .done()
        - win.get_size();

    let scr = dsp.window_get_current_screen_ex().window_id(id).done();
    let use_rect = dsp.screen_get_usable_rect_ex().screen(scr).done();

    let usable_size = use_rect.size - brd_size;
    let Some(scale) = crate::get_largest_integer_scale(min_size, usable_size) else {
        tracing::error!(
            window = %win,
            %min_size,
            %usable_size,
            "could not find integer scale for window within bounds; skipping fit_window()",
        );
        return;
    };

    win.set_size(min_size * scale as i32);
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
    fn largest_integer_scale_fails_on_negative_or_zero_base() {
        assert_lrg!({0, 1}, {1, 1} => None);
        assert_lrg!({1, 0}, {1, 1} => None);
        assert_lrg!({-1, 1}, {1, 1} => None);
        assert_lrg!({1, -1}, {1, 1} => None);
    }

    #[test]
    fn largest_integer_scale_fails_on_negative_bounds() {
        assert_lrg!({1, 1}, {-1, 1} => None);
        assert_lrg!({1, 1}, {1, -1} => None);
    }

    #[test]
    fn largest_integer_scale_picks_smaller_of_either_x_or_y() {
        assert_lrg!({1, 1}, {2, 1} => Some(1));
        assert_lrg!({1, 1}, {1, 2} => Some(1));
        assert_lrg!({1, 1}, {2, 2} => Some(2));
    }
}
