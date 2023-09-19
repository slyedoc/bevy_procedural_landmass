mod camera_controller;

use camera_controller::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    text::DEFAULT_FONT_HANDLE, pbr::wireframe::WireframePlugin,
};
use bevy_atmosphere::prelude::AtmospherePlugin;

pub mod prelude {
    pub use crate::ExampleCommonPlugin;
    pub use crate::camera_controller::CameraController;
    pub use bevy_atmosphere::prelude::AtmosphereCamera;
    
}

pub struct ExampleCommonPlugin;

impl Plugin for ExampleCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin,
            // WorldInspectorPlugin::new(),
            WireframePlugin,
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


