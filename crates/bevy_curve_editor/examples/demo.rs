use bevy::prelude::*;
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin};

use bevy_curve_editor::{BevyCurveEditorPlugin, Curve};

#[derive(Clone, Resource, Reflect, InspectorOptions, Default)]
#[reflect(Resource, InspectorOptions)]
struct Example {
    curve1: Curve,
    curve2: Curve,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            BevyCurveEditorPlugin,
            ResourceInspectorPlugin::<Example>::default(),
        ))
        .init_resource::<Example>()
        //.register_type::<Example>()
        .run();
}
