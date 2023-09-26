mod camera_controller;

use bevy_inspector_egui::{bevy_egui::*, egui, bevy_inspector};
use camera_controller::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    text::DEFAULT_FONT_HANDLE, log::LogPlugin, window::PrimaryWindow,
};
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_procedural_landmass::prelude::*;

pub mod prelude {
    pub use crate::{ExampleCommonPlugin,  toggle_debug_rain, toggle_wireframe, inspector_ui, bevy_log_plugin };
    pub use crate::camera_controller::CameraController;
    pub use bevy_atmosphere::prelude::AtmosphereCamera;    
}

pub struct ExampleCommonPlugin;

impl Plugin for ExampleCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin,
            bevy_inspector_egui::DefaultInspectorConfigPlugin,
            CameraControllerPlugin,
            AtmospherePlugin,
        ))
        .add_systems(Startup, setup)

        .add_systems(Update, update_fps_text);
    }
}


#[derive(Component)]
struct FpsText;

fn setup(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "FPS: ",
            TextStyle {
                font: DEFAULT_FONT_HANDLE.typed(),
                font_size: 20.0,
                color: Color::TOMATO,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        FpsText,
    ));
}

fn update_fps_text(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[0].value = format!("FPS: {value:.2}");
            }
        }
    }
}


pub fn bevy_log_plugin() -> LogPlugin {
    LogPlugin {
        level: bevy::log::Level::TRACE,
        filter: "\
          gilrs_core=info,gilrs=info,\
          naga=warn,wgpu=warn,wgpu_hal=warn,wgpu_core::device=warn,\
          naga::back::spv::writer=warn,\
          bevy_pbr::render::mesh=debug,\
          bevy_app=info,bevy_render::render_resource::pipeline_cache=warn,\
          bevy_render::view::window=warn,bevy_ecs::world::entity_ref=warn"
            .to_string(),
    }
}

pub fn inspector_ui(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    egui::Window::new("UI")
        // .default_size([500.0, 500.0])
        .resizable(true)        
        .show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            bevy_inspector::ui_for_world(world, ui);
             
            egui::CollapsingHeader::new("StandardMaterials").show(ui, |ui| {
                bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
            });
            egui::CollapsingHeader::new("WaterMaterials").show(ui, |ui| {
                bevy_inspector::ui_for_assets::<WaterMaterial>(world, ui);
            });

            // ui.heading("Entities");
            // bevy_inspector::ui_for_world_entities(world, ui);
        });
    });
}

pub fn toggle_wireframe(
    terrain_wireframe: Res<State<TerrainWireframeMode>>,
    mut next_state: ResMut<NextState<TerrainWireframeMode>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Key1) {
        match terrain_wireframe.get() {
            TerrainWireframeMode::On => next_state.set(TerrainWireframeMode::Off),
            TerrainWireframeMode::Off => next_state.set(TerrainWireframeMode::On),
        }
    }
}

pub fn toggle_debug_rain(
    state: Res<State<TerrainDebugRainMode>>,
    mut next_state: ResMut<NextState<TerrainDebugRainMode>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Key2) {
        match state.get() {
            TerrainDebugRainMode::On => next_state.set(TerrainDebugRainMode::Off),
            TerrainDebugRainMode::Off => next_state.set(TerrainDebugRainMode::On),
        }
    }
}
