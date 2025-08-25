<script lang="ts">
  import along from "@turf/along";
  import type { FeatureCollection } from "geojson";
  import { onDestroy } from "svelte";
  import { CircleLayer, GeoJSON } from "svelte-maplibre";
  import { layerId } from "./common";
  import type { AllShortcuts } from "./wasm";

  let { paths }: { paths: AllShortcuts } = $props();

  let totalDirectness = $derived.by(sumWeights);

  let numDots = 50;
  let redrawMs = 100;
  let stepKm = 0.01;

  interface Dot {
    idx: number;
    distance: number;
  }

  let dots = $derived.by(makeDots);
  let gj = $derived.by(redraw);

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
      if (
        dot.distance >=
        paths.features[dot.idx].properties.length_meters / 1000
      ) {
        dots[idx] = startDot();
      }
    }
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
