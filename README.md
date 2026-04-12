# HighwayAreaMap

Visualizes `area:highway` polygons mapped in OpenStreetMap.

Exports in `osm.pbf` format are supported, as available from https://download.geofabrik.de/.

This prototype was developed during https://wiki.openstreetmap.org/wiki/Berlin_Hack_Weekend_April_2026.

## Dependencies

1. `rust`
2. `cargo`
3. `wasm-pack`

## Native

```sh
# Compile as native binary and load "export.osm.pbf" file.
cargo run --release -- export.osm.pbf
```

## WebAssembly

```sh
# Compile as WebAssembly and start local webserver.
wasm-pack build --target web
python3 -m http.server 8000
```

Open `localhost:8000/web` in a browser, drag the `osm.pbf` file in the browser window.
