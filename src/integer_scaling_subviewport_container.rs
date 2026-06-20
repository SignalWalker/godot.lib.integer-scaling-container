use godot::{
    classes::{
        ISubViewportContainer, Node, SubViewport, SubViewportContainer,
        class_macros::private::virtuals::{Xrvrs::Gd, ZipReader::Vector2i},
        notify::ContainerNotification,
        viewport::DefaultCanvasItemTextureFilter,
    },
    obj::{Base, WithBaseField, WithUserSignals},
    register::{
        GodotClass, godot_api,
        info::{PropertyInfo, PropertyUsageFlags},
    },
    signal::ConnectHandle,
};

#[derive(GodotClass)]
#[class(tool, base = SubViewportContainer)]
pub struct IntegerScalingSubViewportContainer {
    base: Base<SubViewportContainer>,

    #[var(
        set,
        hint = LINK,
        hint_string = "suffix:px"
    )]
    #[export]
    base_size: Vector2i,
}

impl IntegerScalingSubViewportContainer {
    fn connect_signals(&mut self) -> ConnectHandle {
        self.signals()
            .child_entered_tree()
            .connect_self(Self::on_child_entered)
    }
    fn on_child_entered(&mut self, child: Gd<Node>) {
        if let Ok(mut child) = child.try_cast::<SubViewport>() {
            child.set_default_canvas_item_texture_filter(DefaultCanvasItemTextureFilter::NEAREST);
            child.set_snap_2d_vertices_to_pixel(true);
            child.set_snap_controls_to_pixels(true);
        }
    }
    fn update_scale(&mut self) {
        self.base_mut().set_stretch(true);
        let within = self.base().get_size().cast_int();
        let Some(scale) = crate::get_largest_integer_scale(self.base_size, within) else {
            tracing::error!(
                base_size = %self.base_size,
                %within,
                container = %self.to_gd(),
                "could not get scale factor",
            );
            return;
        };
        self.base_mut().set_stretch_shrink(scale as i32);
        self.base_mut().queue_sort();
    }
}

#[godot_api]
impl IntegerScalingSubViewportContainer {
    /// HACK :: it seems like you can't use `self.signals()` if you don't have any user defined signals?
    #[signal(internal)]
    fn dummy();

    #[func]
    pub fn set_base_size(&mut self, new: Vector2i) {
        self.base_size = new;
        self.base_mut().set_custom_minimum_size(new.cast_float());
        self.update_scale();
    }
}

#[godot_api]
impl ISubViewportContainer for IntegerScalingSubViewportContainer {
    fn init(base: Base<SubViewportContainer>) -> Self {
        Self {
            base,
            base_size: Vector2i { x: 1, y: 1 },
        }
    }

    fn enter_tree(&mut self) {
        self.base_mut().set_stretch(true);
        self.connect_signals();
    }

    fn on_validate_property(&self, info: &mut PropertyInfo) {
        if info.property_name == "custom_minimum_size"
            || info.property_name == "stretch"
            || info.property_name == "stretch_shrink"
        {
            info.usage |= PropertyUsageFlags::READ_ONLY;
        }
    }

    fn on_notification(&mut self, notif: ContainerNotification) {
        match notif {
            // necessary because this is an @tool class and we connect a signal when we enter_tree
            ContainerNotification::EXTENSION_RELOADED => {
                self.connect_signals();
            }
            ContainerNotification::RESIZED => {
                self.update_scale();
            }
            _ => (),
        }
    }

    fn ready(&mut self) {
        self.update_scale();
    }
}
