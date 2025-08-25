<script lang="ts">
  import turfDistance from "@turf/distance";
  import type { Feature, LineString } from "geojson";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { layerId } from "../common";

  interface Props {
    map: Map;
    onDone: (f: Feature<LineString> | null) => void;
  }

  let { map, onDone }: Props = $props();

  let line: Feature<LineString> | null = $state(null);

  map.on("dragstart", onDragStart);
  map.on("mousemove", onMouseMove);
  map.on("mouseup", onMouseUp);

  onDestroy(() => {
    map.dragPan.enable();
    map.off("dragstart", onDragStart);
    map.off("mousemove", onMouseMove);
    map.off("mouseup", onMouseUp);
  });

  function onDragStart() {
    map.dragPan.disable();
    line = {
      type: "Feature",
      properties: {},
      geometry: {
        type: "LineString",
        coordinates: [],
      },
    };
  }

  function onMouseMove(e: MapMouseEvent) {
    if (line) {
      let next = e.lngLat.toArray();
      let prev = line.geometry.coordinates.at(-1);

      if (prev) {
        let distanceMeters = turfDistance(prev, next) * 1000;
        if (distanceMeters < 0.5) {
          // Skip if the distance is too small, this avoids redraw.
          return;
        }
      }

      line.geometry.coordinates.push(next);
      line = line;
    }
  }

  function onMouseUp() {
    if (!line || line.geometry.coordinates.length == 0) {
      onDone(null);
    } else {
      onDone(line);
    }
    line = null;
    map.dragPan.enable();
  }
</script>

{#if line}
  <GeoJSON data={line}>
    <LineLayer
      {...layerId("freehand-line")}
      paint={{
        "line-width": 5,
        "line-color": "red",
      }}
    />
  </GeoJSON>
{/if}
