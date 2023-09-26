use bevy::{prelude::*, render::render_resource::{AsBindGroup, ShaderRef}, reflect::{TypePath, TypeUuid}, window::PrimaryWindow};
use bevy_inspector_egui::{prelude::*, inspector_options::std_options::NumberDisplay};

//use crate::egui_helper::insert_options_struct;

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WaterMaterial> {
            // This material only needs to read the prepass textures,
            // but the meshes using it should not contribute to the prepass render, so we can disable it.
            prepass_enabled: true,
            ..default()
        })        
        .register_type::<WaterMaterial>();
    }
}


#[derive(AsBindGroup, TypeUuid, Debug, Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
#[uuid = "4d557c50-4907-4ed8-9bb3-e8a699bee5aa"]
pub struct WaterMaterial {
    #[uniform(0)]
    shallow_color: Color,
    #[uniform(0)]
    deep_color: Color,
    #[uniform(0)]
    edge_color: Color,
    #[uniform(0)]
    edge_scale: f32,
    #[uniform(0)]
    #[inspector(min = 0.0, max = 1.0,  speed = 0.01, display = NumberDisplay::Slider)]
    water_clarity: f32,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    //pub alpha_mode: AlphaMode,
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            color_texture: None,
            //alpha_mode: AlphaMode::Blend,
            shallow_color: Color::rgba(0.0, 0.5, 1.0, 0.5),
            deep_color: Color::rgba(0.0, 0.2, 0.4, 1.0),
            edge_color: Color::WHITE,
            edge_scale: 10.0,
            water_clarity: 0.05,
        }
    }
}

/// Not shown in this example, but if you need to specialize your material, the specialize
/// function will also be used by the prepass
impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    // You can override the default shaders used in the prepass if your material does
    // anything not supported by the default prepass
    // fn prepass_fragment_shader() -> ShaderRef {
    //     "shaders/custom_material.wgsl".into()
    // }
}
