use godot::{
    classes::{
        Container, Control, IContainer, Window,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{Rect2, Vector2i},
        },
        notify::ContainerNotification,
    },
    obj::{Base, WithBaseField},
    register::{
        GodotClass, godot_api,
        info::{PropertyInfo, PropertyUsageFlags},
    },
};

/// A [Container] that resizes its child elements to the largest integer multiple of a minimum size
/// that fits within the container.
#[derive(GodotClass)]
#[class(tool, base = Container)]
pub struct IntegerScalingContainer {
    base: Base<Container>,

    /// The minimum element size.
    /// Both components must be greater than 0.
    #[var(
        set,
        hint = LINK,
        hint_string = "suffix:px"
    )]
    #[export]
    base_size: Vector2i,
}

impl IntegerScalingContainer {
    /// Get the largest scale by which the base size of this container may be multiplied and still
    /// fit within the container.
    pub fn get_largest_scale(&self) -> Option<u32> {
        crate::get_largest_integer_scale(self.base_size, self.base().get_size().cast_int())
    }
    /// Get the largest integer-scaled [Rect2] that fits within this container.
    pub fn get_scaled_rect(&self) -> Option<Rect2> {
        let scale = match self.get_largest_scale() {
            None | Some(0) => return None,
            Some(scale) => scale as f32,
        };
        let new_size = self.base_size.cast_float() * scale;
        Some(Rect2::new(
            self.base().get_size() / 2.0 - new_size / 2.0,
            new_size,
        ))
    }
}

#[godot_api]
impl IContainer for IntegerScalingContainer {
    fn init(base: Base<Container>) -> Self {
        Self {
            base,
            base_size: Vector2i { x: 1, y: 1 },
        }
    }

    fn on_validate_property(&self, info: &mut PropertyInfo) {
        if info.property_name == "custom_minimum_size" {
            info.usage |= PropertyUsageFlags::READ_ONLY
        }
    }

    fn on_notification(&mut self, notif: ContainerNotification) {
        if notif == ContainerNotification::SORT_CHILDREN {
            // resize child controls to fit within our new bounds
            let Some(new_rect) = self.get_scaled_rect() else {
                tracing::error!(
                    base_size = %self.base_size,
                    actual_size = %self.base().get_size(),
                    container = %self.to_gd(),
                    "could not get scaled rect",
                );
                return;
            };
            for node in self.base().get_children().iter_shared() {
                if let Ok(c) = node.try_cast::<Control>() {
                    self.base_mut().fit_child_in_rect(&c, new_rect);
                }
            }
        }
    }
}

#[godot_api]
impl IntegerScalingContainer {
    /// Set the base size of elements within this container.
    ///
    /// This will also set the minimum size of this control, and will queue a resort of contained controls.
    #[func]
    pub fn set_base_size(&mut self, value: Vector2i) {
        if value.x < 1 || value.y < 1 {
            tracing::error!(
                base_size = %value,
                "invalid integer scaling base size (both components must be at least 1)",
            );
            return;
        }
        self.base_size = value;
        self.base_mut().set_custom_minimum_size(value.cast_float());
        self.base_mut().queue_sort();
    }

    /// Resize the given window to be the largest integer multiple of the given minimum size that
    /// fits within its current screen.
    #[func]
    fn fit_window(mut win: Gd<Window>, min_size: Vector2i) {
        crate::fit_window(&mut win, min_size);
    }
}
