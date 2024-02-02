<script lang="ts">
  import type { Feature, LineString } from "geojson";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { createEventDispatcher, onDestroy } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { layerId } from "../common";

  export let map: Map;
  let line: Feature<LineString> | null = null;

  const dispatch = createEventDispatcher<{
    done: Feature<LineString> | null;
  }>();

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
      // TODO Simplify
      line.geometry.coordinates.push(e.lngLat.toArray());
      line = line;
    }
  }

  function onMouseUp() {
    if (line) {
      dispatch("done", line);
      line = null;
    }
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
