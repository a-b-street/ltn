import type { FeatureCollection } from "geojson";

// Sets a 'color' property on any cell polygons. Idempotent.
export function setCellColors(gj: FeatureCollection): FeatureCollection {
  // A qualitative palette from colorbrewer2.org, skipping the red hue (used
  // for levels of shortcutting) and grey (too close to the basemap)
  let cell_colors = [
    "#8dd3c7",
    "#ffffb3",
    "#bebada",
    "#80b1d3",
    "#fdb462",
    "#b3de69",
    "#fccde5",
    "#bc80bd",
    "#ccebc5",
    "#ffed6f",
  ];

  for (let f of gj.features) {
    f.properties ??= {};
    if (f.properties.cell_color == "disconnected") {
      f.properties.color = "red";
    } else if (Object.hasOwn(f.properties, "cell_color")) {
      f.properties.color =
        cell_colors[f.properties.cell_color % cell_colors.length];
    }
  }

  return gj;
}
