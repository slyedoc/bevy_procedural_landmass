use core::panic;
use std::any::{Any, TypeId};

use bevy::prelude::*;
use bevy_inspector_egui::{
    inspector_egui_impls::InspectorEguiImpl,  reflect_inspector::InspectorUi, *,
};
use pretty_type_name::pretty_type_name;

pub struct BevyCurveEditorPlugin;

impl Plugin for BevyCurveEditorPlugin {
    fn build(&self, app: &mut App) {

        app.register_type::<Curve>();
        
        let type_registry = app.world.resource::<AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        type_registry.get_mut(TypeId::of::<Curve>())
            .unwrap_or_else(|| panic!("{:?} not registered", std::any::type_name::<Curve>()))
            .insert(InspectorEguiImpl::new(
                curve_ui,
                curve_ui_readonly,
                many_unimplemented::<Curve>,
            ));                


        //     "Curve",
        //     InspectorType::new::<Curve>()
        //         .ui(curve_ui)
        //         .readonly_ui(curve_ui_readonly),
        // let i = InspectorEguiImpl::new(
        //     curve_ui,
        //     curve_ui_readonly,
        //     many_unimplemented::<Curve>,
        // );
        // let res = app.world.resource_mut::<AppTypeRegistry>();
        // let mut r = res.write();
        // app.register_type::


        // r.get_mut(TypeId::of::<Curve>())
        //     .insert);
    }
}

#[derive(Clone, Reflect, Default)]
pub struct Curve {
    selected_keyframe: usize,
}

fn curve_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let curve = value.downcast_ref::<Curve>().unwrap();
    ui.label(format!("curve: {}", curve.selected_keyframe));
    false
}

fn curve_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) {
    let curve = value.downcast_ref::<Curve>().unwrap();
    ui.label(format!("curve readonly: {}", curve.selected_keyframe));
}

fn many_unimplemented<T: Any>(
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn Reflect],
    _projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> bool {
    no_multiedit(ui, &pretty_type_name::<T>());
    false
}

pub fn no_multiedit(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (egui::FontId::monospace(12.0), type_name),
        (
            egui::FontId::proportional(13.0),
            " doesn't support multi-editing.",
        ),
    ]);

    ui.label(job);
}

pub fn layout_job(text: &[(egui::FontId, &str)]) -> egui::epaint::text::LayoutJob {
    let mut job = egui::epaint::text::LayoutJob::default();
    for (font_id, text) in text {
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                ..Default::default()
            },
        );
    }
    job
}
