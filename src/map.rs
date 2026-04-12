use bevy::math::Vec2;
use bevy::{color::palettes::css::*, prelude::*};
use bevy_prototype_lyon::prelude::*;

pub struct Geometry {
    pub polyline: Vec<Vec2>,
    pub closed: bool,
    pub layer: Option<i32>,
}

pub struct Kerb {
    pub geometry: Geometry,
}

pub struct Building {
    pub geometry: Geometry,
}

pub struct HighwayArea {
    pub geometry: Geometry,
    pub highway: Option<String>,
    pub surface: Option<String>,
}

pub enum CrossingType {
    Zebra,
    DashedLines,
}

pub struct HighwayCrossing {
    pub geometry: Geometry,
    pub category: Option<CrossingType>,
}

pub enum MapObject {
    Kerb(Kerb),
    HighwayCrossing(HighwayCrossing),
    HighwayArea(HighwayArea),
    Building(Building),
}

pub fn spawn_object(commands: &mut Commands, object: &MapObject) {
    match &object {
        MapObject::HighwayArea(area) => {
            let lyon_polygon = shapes::Polygon {
                points: area.geometry.polyline.clone(),
                closed: area.geometry.closed,
            };

            let color = if let Some(surface) = area.surface.as_ref() {
                match surface.as_str() {
                    "paving_stones" => GREY,
                    "sett" => DARK_SLATE_GRAY,
                    "grass" => GREEN,
                    _ => DARK_KHAKI,
                }
            } else {
                area.highway
                    .as_ref()
                    .map_or(BLACK, |value| match value.as_str() {
                        "footway" | "service" => GRAY,
                        "cycleway" => RED,
                        "traffic_island" => DARK_GRAY,
                        "street_side" => LIGHT_STEEL_BLUE,
                        _ => DARK_KHAKI,
                    })
            };

            let order = area
                .highway
                .as_ref()
                .map_or(0.0, |value| match value.as_str() {
                    "footway" => 2.0,
                    "cycleway" => 1.0,
                    "traffic_island" => 0.0,
                    _ => -2.0,
                });

            commands.spawn((
                ShapeBuilder::with(&lyon_polygon).fill(color).build(),
                Transform::from_xyz(
                    0.,
                    0.,
                    area.geometry
                        .layer
                        .map_or(order, |value| value as f32 + order),
                ),
            ));
        }

        MapObject::Kerb(kerb) => {
            let lyon_polygon = shapes::Polygon {
                points: kerb.geometry.polyline.clone(),
                closed: kerb.geometry.closed,
            };

            commands.spawn((
                ShapeBuilder::with(&lyon_polygon)
                    .stroke((BLACK, 0.02))
                    .build(),
                Transform::from_xyz(
                    0.,
                    0.,
                    kerb.geometry
                        .layer
                        .map_or(10.0, |value| value as f32 + 10.0),
                ),
            ));
        }

        MapObject::HighwayCrossing(crossing) => {
            if let Some(_) = &crossing.category {
                let a = crossing.geometry.polyline.last().unwrap();
                let b = crossing.geometry.polyline.first().unwrap();
                let normal = (a - b).normalize();
                let tangent = Rot2::FRAC_PI_2 * normal;

                let half_width = 0.05;
                let left = shapes::Line(a + half_width * tangent, b + half_width * tangent);
                let right = shapes::Line(a - half_width * tangent, b - half_width * tangent);

                commands.spawn(
                    ShapeBuilder::with(&left)
                        .stroke((Color::WHITE, 0.02))
                        .build(),
                );

                commands.spawn(
                    ShapeBuilder::with(&right)
                        .stroke((Color::WHITE, 0.02))
                        .build(),
                );
            }
        }

        MapObject::Building(building) => {
            let lyon_polygon = shapes::Polygon {
                points: building.geometry.polyline.clone(),
                closed: building.geometry.closed,
            };

            commands.spawn((
                ShapeBuilder::with(&lyon_polygon).fill(LIGHT_SALMON).build(),
                Transform::from_xyz(
                    0.,
                    0.,
                    building.geometry.layer.map_or(0.0, |value| value as f32),
                ),
            ));
        }
    }
}
