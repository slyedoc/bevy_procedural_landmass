
use std::any::Any;

use bevy::prelude::*;
use bevy_inspector_egui::{
    reflect_inspector::InspectorUi, *,
};
use pretty_type_name::pretty_type_name;


pub fn many_unimplemented<T: Any>(
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

fn no_multiedit(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (egui::FontId::monospace(12.0), type_name),
        (
            egui::FontId::proportional(13.0),
            " doesn't support multi-editing.",
        ),
    ]);

    ui.label(job);
}

fn layout_job(text: &[(egui::FontId, &str)]) -> egui::epaint::text::LayoutJob {
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

// pub fn insert_options_struct<T: 'static>(
//     type_registry: & AppTypeRegistry,
//     fields: &[(&'static str, &dyn TypeData)],
// ) {

//     let mut type_registry = type_registry.write();

//     let Some(registration) = type_registry.get_mut(std::any::TypeId::of::<T>()) else {
//         warn!("Attempting to set default inspector options for {}, but it wasn't registered in the type registry.", std::any::type_name::<T>());
//         return;
//     };
//     if registration.data::<ReflectInspectorOptions>().is_none() {
//         let mut options = InspectorOptions::new();
//         for (field, data) in fields {
//             let info = match registration.type_info() {
//                 TypeInfo::Struct(info) => info,
//                 _ => unreachable!(),
//             };
//             let field_index = info.index_of(field).unwrap();
//             options.insert_boxed(Target::Field(field_index), TypeData::clone_type_data(*data));
//         }
//         registration.insert(ReflectInspectorOptions(options));
//     }
// }