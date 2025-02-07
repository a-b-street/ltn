<script lang="ts">
  import type { Feature, FeatureCollection, MultiPolygon } from "geojson";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
  } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link, sum } from "./common";
  import { backend, mode } from "./stores";
  import type { ZoneDemandProps } from "./wasm";

  let gj = emptyGeojson() as FeatureCollection<MultiPolygon, ZoneDemandProps>;
  try {
    gj = $backend!.getDemandModel();
    console.log(gj);
  } catch (err) {
    window.alert("No demand model for this area");
    $mode = { mode: "pick-neighbourhood" };
  }

  let hovered: Feature | null = null;
  $: hoveredId = hovered == null ? null : (hovered.id as number);
  // MapLibre doesn't preserve the arrays in properties, so use the original version
  $: current = hoveredId != null ? gj.features[hoveredId] : null;
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "pick-neighbourhood" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>Debug demand model</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "pick-neighbourhood" })} />

    <p>{gj.features.length.toLocaleString()} zones</p>

    {#if current && hoveredId != null}
      <u>{current.properties.name}</u>
      <p>
        Total trips from here: {sum(
          current.properties.counts_from,
        ).toLocaleString()}
      </p>
      <p>
        Total trips to here: {sum(
          current.properties.counts_to,
        ).toLocaleString()}
      </p>
      <p>
        Total intra-zonal trips starting and ending here: {current.properties
          .counts_from[hoveredId]}
      </p>
    {:else}
      <p>Hover on a zone</p>
    {/if}
  </div>

  <div slot="map">
    <GeoJSON data={gj} generateId>
      <FillLayer
        {...layerId("debug-demand-fill")}
        paint={{
          "fill-color": "grey",
          "fill-opacity": hoverStateFilter(0.5, 0.1),
        }}
        manageHoverState
        bind:hovered
      />

      <LineLayer
        {...layerId("debug-demand-outline")}
        paint={{
          "line-width": 2,
          "line-color": "black",
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
