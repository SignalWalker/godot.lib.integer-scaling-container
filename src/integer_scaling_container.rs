use godot::{
    classes::{
        Container, Control, DisplayServer, Engine, IContainer, Window,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{Rect2, Vector2i},
        },
        notify::ContainerNotification,
    },
    global::godot_error,
    obj::{Base, Singleton, WithBaseField},
    register::{
        GodotClass, godot_api,
        info::{PropertyInfo, PropertyUsageFlags},
    },
};

#[derive(GodotClass)]
#[class(tool, base = Container)]
pub struct IntegerScalingContainer {
    base: Base<Container>,
    #[export]
    #[var(set)]
    base_size: Vector2i,
}

impl IntegerScalingContainer {
    pub fn get_largest_scale(&self) -> Option<u32> {
        crate::get_largest_integer_scale(self.base_size, self.base().get_size().cast_int())
    }
    pub fn get_scaled_rect(&self) -> Option<Rect2> {
        let new = self.base_size.cast_float() * self.get_largest_scale()? as f32;
        Some(Rect2::new(self.base().get_size() / 2.0 - new / 2.0, new))
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
            let Some(new_rect) = self.get_scaled_rect() else {
                godot_error!(
                    "[IntegerViewport] could not get scaled rect for viewport with actual size {} and unscaled size {}",
                    self.base().get_size(),
                    self.base_size
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
    #[func]
    pub fn set_base_size(&mut self, value: Vector2i) {
        if value.x < 1 || value.y < 1 {
            godot_error!(
                "[IntegerViewport] invalid integer scaling base size: {}",
                value
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
    pub fn fit_window(mut win: Gd<Window>, min_size: Vector2i) {
        if win.is_embedded() {
            // TODO :: why is this bad (and is this fixed by that one change in 4.7?)
            godot_error!(
                "[IntegerViewport] called fit_window on an embedded window: {}",
                win
            );
            return;
        }
        if Engine::singleton().is_embedded_in_editor() {
            // TODO :: why
            return;
        }

        win.set_min_size(min_size);

        let dsp = DisplayServer::singleton();

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
            godot_error!(
                "[IntegerViewport] could not find integer scale for {} within {}; skipping fit_window({})",
                min_size,
                usable_size,
                win
            );
            return;
        };

        win.set_size(min_size * scale as i32);
    }
}
