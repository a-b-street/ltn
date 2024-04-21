<script lang="ts">
  import turfLength from "@turf/length";
  import along from "@turf/along";
  import { layerId } from "./common";
  import type { FeatureCollection, LineString } from "geojson";
  import { GeoJSON, CircleLayer } from "svelte-maplibre";
  import { onDestroy } from "svelte";

  export let paths: FeatureCollection<LineString, { directness: number }>;
  let totalDirectness = sumWeights();

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
    totalDirectness = sumWeights();
    dots = makeDots();
  }

  let intervalId = setInterval(animate, redrawMs);
  onDestroy(() => clearInterval(intervalId));

  function sumWeights(): number {
    // Small directness is better, so invert
    return paths.features
      .map((f) => 1 / f.properties.directness)
      .reduce((t, n) => t + n, 0);
  }

  function makeDots(): Dot[] {
    if (paths.features.length == 0) {
      return [];
    }
    return Array.from({ length: numDots }, startDot);
  }

  function startDot(): Dot {
    // Weighted random sample
    let rand = Math.random() * totalDirectness;
    let cumulativeWeight = 0;
    for (let [idx, path] of paths.features.entries()) {
      cumulativeWeight += 1 / path.properties.directness;
      if (rand < cumulativeWeight) {
        return {
          idx,
          length: turfLength(path, { units: "kilometers" }),
          distance: 0,
        };
      }
    }
    throw new Error(`didnt pick dot, totalDirectness is ${totalDirectness}`);
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
