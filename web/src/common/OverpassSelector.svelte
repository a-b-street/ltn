<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import type { LngLat, Map } from "maplibre-gl";
  import { createEventDispatcher } from "svelte";
  import { overpassQueryForPolygon } from "./";
  import { PolygonTool } from "./draw_polygon/polygon_tool";
  import PolygonControls from "./draw_polygon/PolygonControls.svelte";

  export let map: Map | null;

  const dispatch = createEventDispatcher<{
    loading: string;
    gotXml: string;
    error: string;
  }>();

  let polygonTool: PolygonTool | null = null;

  async function importPolygon(boundaryGj: Feature<Polygon>) {
    try {
      dispatch("loading", "Loading from Overpass");
      let resp = await fetch(overpassQueryForPolygon(boundaryGj));
      let osmXml = await resp.text();

      dispatch("gotXml", osmXml);
    } catch (err: any) {
      dispatch("error", err.toString());
    }
  }

  function latLngToGeojson(pt: LngLat): [number, number] {
    return [pt.lng, pt.lat];
  }

  // Turn the current viewport into a rectangular boundary
  function mapBoundsToGeojson(): Feature<Polygon> {
    let b = map!.getBounds();
    return {
      type: "Feature",
      properties: {},
      geometry: {
        coordinates: [
          [
            latLngToGeojson(b.getSouthWest()),
            latLngToGeojson(b.getNorthWest()),
            latLngToGeojson(b.getNorthEast()),
            latLngToGeojson(b.getSouthEast()),
            latLngToGeojson(b.getSouthWest()),
          ],
        ],
        type: "Polygon",
      },
    };
  }

  async function importCurrentView() {
    if (!map) {
      return;
    }
    if (map.getZoom() < 13) {
      dispatch("error", "Zoom in more to import");
      return;
    }
    await importPolygon(mapBoundsToGeojson());
  }

  function startPolygonTool() {
    if (!map) {
      return;
    }
    polygonTool = new PolygonTool(map);
    polygonTool.startNew();
    polygonTool.addEventListenerSuccess(async (f) => {
      polygonTool = null;
      await importPolygon(f);
    });
    polygonTool.addEventListenerFailure(() => {
      polygonTool = null;
    });
  }
</script>

{#if polygonTool}
  <PolygonControls {polygonTool} />
{:else}
  <div>
    <button type="button" on:click={importCurrentView}
      >Import current view</button
    >
  </div>

  <i>or...</i>

  <div>
    <button type="button" on:click={startPolygonTool}
      >Draw an area to import on the map</button
    >
  </div>
{/if}
