pub mod map;
pub mod osm;

use bevy::{color::palettes::css::*, prelude::*};
use bevy_pancam::*;
use bevy_prototype_lyon::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct HighwayAreaMap {
    data: Option<Vec<u8>>,
}

#[derive(Resource)]
struct DataResource {
    data: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl HighwayAreaMap {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Option<Vec<u8>>) -> Self {
        Self {
            data: data.map(|array| array.to_vec()),
        }
    }

    #[wasm_bindgen]
    pub fn run(&mut self) {
        console_error_panic_hook::set_once();

        App::new()
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bevy-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }))
            .add_plugins(ShapePlugin)
            .add_plugins(PanCamPlugin::default())
            .insert_resource(DataResource {
                data: self.data.clone(),
            })
            .insert_resource(ClearColor(Color::Srgba(LIGHT_SLATE_GRAY)))
            .add_systems(Startup, setup)
            .run();
    }
}

fn setup(mut commands: Commands, data_resource: Res<DataResource>) {
    commands.spawn((Camera2d, PanCam::default()));

    if let Some(data) = &data_resource.data {
        if let Ok(objects) = osm::load_map_objects(&data) {
            objects
                .iter()
                .for_each(|object| map::spawn_object(&mut commands, object));
        }
    }
}
