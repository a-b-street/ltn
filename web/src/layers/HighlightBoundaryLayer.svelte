<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { FillLayer, GeoJSON } from "svelte-maplibre";
  import { layerId } from "../common";
  import type { RenderNeighbourhoodOutput } from "../wasm";

  export let gj: RenderNeighbourhoodOutput;

  function invertBoundary(gj: RenderNeighbourhoodOutput): Feature<Polygon> {
    // @ts-expect-error TS can't figure out that we're narrowing the case here
    let boundary: Feature<Polygon> = gj.features.find(
      (f) => f.properties.kind == "boundary",
    )!;

    return {
      type: "Feature",
      properties: {},
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [180.0, 90.0],
            [-180.0, 90.0],
            [-180.0, -90.0],
            [180.0, -90.0],
            [180.0, 90.0],
          ],
          // One hole
          boundary.geometry.coordinates[0],
        ],
      },
    };
  }
</script>

<GeoJSON data={invertBoundary(gj)}>
  <FillLayer
    {...layerId("neighbourhood-boundary")}
    paint={{ "fill-color": "black", "fill-opacity": 0.3 }}
  />
</GeoJSON>
