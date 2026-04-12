use crate::map::*;
use bevy::math::Vec2;
use osmpbfreader::{Node, NodeId, Way};
use std::collections::{HashMap, HashSet};
use std::f32;
use std::io::{BufReader, Cursor};

type Ways = Vec<Way>;
type Nodes = HashMap<NodeId, Node>;

pub fn normalize_nodes(nodes: &mut Nodes) {
    let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
    let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

    for point in nodes
        .values()
        .map(|coordinates| Vec2::new(coordinates.lon() as f32, coordinates.lat() as f32))
    {
        min = min.min(point);
        max = max.max(point);
    }

    let center_x = (min.x + max.x) / 2.0;
    let center_y = (min.y + max.y) / 2.0;

    for point in nodes.values_mut() {
        point.decimicro_lon -= (center_x / 1e-7) as i32;
        point.decimicro_lat -= (center_y / 1e-7) as i32;
    }
}

fn ways_and_nodes<Filter>(
    storage: &mut osmpbfreader::OsmPbfReader<BufReader<Cursor<&[u8]>>>,
    filter: Filter,
) -> osmpbfreader::Result<(Ways, Nodes)>
where
    Filter: Fn(&Way) -> bool,
{
    let mut ways = Vec::new();
    let mut needed_nodes: HashSet<NodeId> = HashSet::new();

    for way in storage
        .iter_ways()
        .filter_map(|way| way.ok())
        .filter(filter)
    {
        needed_nodes.extend(&way.nodes);
        ways.push(way);
    }

    storage.rewind()?;

    let mut nodes: HashMap<_, _> = storage
        .iter_nodes()
        .filter_map(|node| node.ok())
        .filter(|node| needed_nodes.contains(&node.id))
        .map(|node| (node.id, node))
        .collect();

    normalize_nodes(&mut nodes);

    Ok((ways, nodes))
}

fn way_to_polygon(way: &Way, nodes: &Nodes) -> Option<(Vec<Vec2>, bool)> {
    let mut coords: Vec<Vec2> = way
        .nodes
        .iter()
        .filter_map(|id| nodes.get(id))
        .map(|node| Vec2 {
            x: (node.lon() * 4000.0) as f32,
            y: (node.lat() * 4000.0) as f32,
        })
        .collect();

    if coords.is_empty() {
        return None;
    }

    let closed = way.nodes.first() == way.nodes.last();

    if closed {
        coords.pop();
    }

    Some((coords, closed))
}

fn crossing_from_string(markings: &str) -> Option<CrossingType> {
    match markings {
        "zebra" => Some(CrossingType::Zebra),
        "dashes" => Some(CrossingType::DashedLines),
        _ => None,
    }
}

pub fn load_map_objects(data: &[u8]) -> Result<Vec<MapObject>, osmpbfreader::error::Error> {
    let cursor = Cursor::new(data);
    let buffer = BufReader::new(cursor);

    let mut reader = osmpbfreader::OsmPbfReader::new(buffer);

    let (ways, nodes) = ways_and_nodes(&mut reader, |way| {
        way.tags.contains_key("area:highway")
            || way.tags.contains("barrier", "kerb")
            || way.tags.contains("footway", "crossing")
            || way.tags.contains("parking", "street_side")
        // || way.tags.contains_key("building")
    })?;

    let objects: Vec<_> = ways
        .iter()
        .filter_map(|way| {
            way_to_polygon(&way, &nodes)
                .map(|(polyline, closed)| {
                    let geometry = || Geometry {
                        polyline: polyline,
                        closed: closed,
                        layer: way
                            .tags
                            .get("layer")
                            .map(|entry| entry.parse::<i32>().ok())
                            .flatten(),
                    };

                    if way.tags.contains_key("area:highway") {
                        return Some(MapObject::HighwayArea(crate::map::HighwayArea {
                            geometry: geometry(),
                            highway: way.tags.get("area:highway").map(|entry| entry.to_string()),
                            surface: way
                                .tags
                                .get("surface")
                                .or(way.tags.get("landuse"))
                                .map(|entry| entry.to_string()),
                        }));
                    } else if way.tags.contains("amenity", "parking")
                        && way.tags.contains("parking", "street_side")
                    {
                        return Some(MapObject::HighwayArea(HighwayArea {
                            geometry: geometry(),
                            highway: "street_side".to_string().into(),
                            surface: way.tags.get("surface").map(|entry| entry.to_string()),
                        }));
                    } else if way.tags.contains("barrier", "kerb") {
                        return Some(MapObject::Kerb(Kerb {
                            geometry: geometry(),
                        }));
                    } else if way.tags.contains("footway", "crossing") {
                        return Some(MapObject::HighwayCrossing(HighwayCrossing {
                            geometry: geometry(),
                            category: way
                                .tags
                                .get("crossing:markings")
                                .map(|value| crossing_from_string(value))
                                .flatten(),
                        }));
                    } else if way.tags.contains_key("building") {
                        return Some(MapObject::Building(Building {
                            geometry: geometry(),
                        }));
                    }

                    None
                })
                .flatten()
        })
        .collect();

    Ok(objects)
}
