<script lang="ts">
  import {
    CircleLayer,
    FillLayer,
    GeoJSON,
    LineLayer,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { emptyGeojson } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, ModeLink, pageTitle, PrevNext } from "./common";
  import { backend, mode } from "./stores";

  let currentOsm: string | null = null;
  let movements = emptyGeojson();
  let idx = 0;

  function pickIntersection(e: LayerClickInfo) {
    currentOsm = e.features[0].properties!.osm;
    movements = $backend!.getMovements(e.features[0].id as number);
    idx = 0;
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <ModeLink mode={{ mode: "title" }} />
        </li>
        <li>
          <ModeLink mode={{ mode: "pick-neighbourhood" }} />
        </li>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton mode={{ mode: "pick-neighbourhood" }} />

    <p>Purple intersections have some kind of turn restriction.</p>

    {#if movements.features.length > 0}
      <button class="secondary" onclick={() => (movements = emptyGeojson())}>
        Pick another intersection
      </button>

      <PrevNext list={movements.features} bind:idx />
    {/if}
    {#if currentOsm}
      <a href={currentOsm} target="_blank">Open OSM</a>
    {/if}
  </div>

  <div slot="map">
    <GeoJSON data={notNull($backend).getAllIntersections()} generateId>
      <CircleLayer
        {...layerId("debug-intersections")}
        paint={{
          "circle-radius": 15,
          "circle-color": [
            "case",
            ["get", "has_turn_restrictions"],
            "purple",
            "black",
          ],
        }}
        manageHoverState
        hoverCursor="pointer"
        onclick={pickIntersection}
      />
    </GeoJSON>

    <GeoJSON data={movements} generateId>
      <LineLayer
        {...layerId("debug-movements-outline")}
        paint={{
          "line-width": 2,
          "line-color": "red",
        }}
      />

      <FillLayer
        {...layerId("debug-movements-fill")}
        filter={["==", ["id"], idx]}
        paint={{
          "fill-color": "cyan",
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
