use std::sync::Mutex;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        query::With,
        reflect::ReflectResource,
        schedule::{BoxedCondition, Condition, IntoSystemConfigs},
        system::{IntoSystem, Resource},
        world::World,
    },
    reflect::{std_traits::ReflectDefault, Reflect, TypePath},
    utils::HashMap,
    window::PrimaryWindow,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin},
    bevy_inspector::{self},
    egui::{self, Context, Ui},
    DefaultInspectorConfigPlugin,
};

#[derive(Default)]
pub struct InspectorPlugin {
    condition: Mutex<Option<BoxedCondition>>,
}

impl InspectorPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }

        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        if app.world.get_resource::<InspectorWindows>().is_none() {
            app.init_resource::<InspectorWindows>();
        }

        let condition = self.condition.lock().unwrap().take();
        let mut system = world_inspector_ui.into_configs();
        if let Some(condition) = condition {
            system.run_if_dyn(condition);
        }
        app.add_systems(Update, system);
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct InspectorWindows {
    windows: HashMap<WindowName, bool>,
}

#[derive(Reflect, Hash, PartialEq, Eq)]
pub struct WindowName(&'static str);

impl WindowName {
    pub fn from_type_path<T: TypePath>() -> Self {
        Self(T::short_type_path())
    }

    pub fn get(&self) -> &'static str {
        self.0
    }
}

pub fn show_window<T: TypePath, R>(
    window_context: &mut InspectorWindows,
    egui_context: &Context,
    add_contents: impl FnOnce(&mut Ui) -> R,
) {
    match window_context
        .windows
        .get_mut(&WindowName::from_type_path::<T>())
    {
        Some(state) => {
            egui::Window::new(T::short_type_path())
                .open(state)
                .show(egui_context, add_contents);
        }
        None => {
            egui::Window::new(T::short_type_path()).show(egui_context, |ui| {
                ui.label(format!(
                    "Cannot find {} registered in the `InspectorWindows`",
                    T::short_type_path()
                ))
            });
        }
    }
}

fn ui_for_inspector_windows(world: &mut World, ui: &mut Ui) {
    let Some(mut windows) = world.get_resource_mut::<InspectorWindows>() else {
        ui.label("Cannot find `InspectorWindows`. Try adding `InspectorPlugin`");
        return;
    };

    for (name, state) in windows.windows.iter_mut() {
        ui.checkbox(state, name.0);
    }
}

pub trait InspectorWindowsAppExt {
    fn register_window<T: TypePath>(&mut self) -> &mut Self;
}

impl InspectorWindowsAppExt for App {
    fn register_window<T: TypePath>(&mut self) -> &mut Self {
        match self.world.get_resource_mut::<InspectorWindows>() {
            Some(mut res) => {
                res.windows.insert(WindowName::from_type_path::<T>(), false);
            }
            None => {
                panic!("Cannot find `InspectorWindows` added. `register_window()` must be called after adding `InspectorPlugin`")
            }
        }
        self
    }
}

fn world_inspector_ui(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::SidePanel::left("world_inspector_left_panel")
        .resizable(true)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                egui::CollapsingHeader::new("Windows")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui_for_inspector_windows(world, ui);
                    });
                ui.separator();
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}
