<script lang="ts">
  import turfLength from "@turf/length";
  import along from "@turf/along";
  import { layerId } from "./common";
  import type { FeatureCollection, LineString } from "geojson";
  import { GeoJSON, CircleLayer } from "svelte-maplibre";
  import { onDestroy } from "svelte";

  export let paths: FeatureCollection<LineString>;

  // TODO Decrease based on number of paths with high directness
  let numDots = 50;
  let redrawMs = 100;
  let stepKm = 0.01;

  interface Dot {
    idx: number;
    length: number;
    distance: number;
  }

  let dots = makeDots();
  let gj = redraw();

  $: if (paths) {
    dots = makeDots();
  }

  let intervalId = setInterval(animate, redrawMs);
  onDestroy(() => clearInterval(intervalId));

  function makeDots(): Dot[] {
    if (paths.features.length == 0) {
      return [];
    }
    return Array.from({ length: numDots }, startDot);
  }

  function startDot(): Dot {
    let idx = Math.floor(Math.random() * paths.features.length);
    return {
      idx,
      length: turfLength(paths.features[idx], { units: "kilometers" }),
      distance: 0,
    };
  }

  function redraw(): FeatureCollection {
    return {
      type: "FeatureCollection",
      features: dots.map(({ idx, distance }) =>
        along(paths.features[idx], distance, {
          units: "kilometers",
        }),
      ),
    };
  }

  function animate() {
    for (let [idx, dot] of dots.entries()) {
      dot.distance += stepKm;
      if (dot.distance >= dot.length) {
        dots[idx] = startDot();
      }
    }
    gj = redraw();
  }
</script>

<GeoJSON data={gj}>
  <CircleLayer
    {...layerId("animate-shortcuts")}
    paint={{
      "circle-radius": 10,
      "circle-color": "purple",
      "circle-stroke-color": "black",
      "circle-stroke-width": 1,
    }}
  />
</GeoJSON>
