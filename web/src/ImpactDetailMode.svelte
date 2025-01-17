<script lang="ts">
  import type { Feature, FeatureCollection } from "geojson";
  import { onDestroy, onMount } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { DotMarker, gjPosition, layerId, Link } from "./common";
  import { ModalFilterLayer } from "./layers";
  import { backend, map, mode } from "./stores";

  export let road: Feature;

  // TODO Weird to modify the input like this?
  road.properties!.kind = "focus";

  let routes = $backend!.getImpactsOnRoad(road.properties!.id);
  let idx = 0;

  onMount(() => {
    $map?.keyboard.disable();
  });
  onDestroy(() => {
    $map?.keyboard.enable();
  });

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "ArrowLeft") {
      e.stopPropagation();
      prev();
    }
    if (e.key == "ArrowRight") {
      e.stopPropagation();
      next();
    }
  }

  function prev() {
    if (idx != 0) {
      idx--;
    }
  }

  function next() {
    if (idx != routes.length - 1) {
      idx++;
    }
  }

  function gj(idx: number): FeatureCollection {
    return {
      type: "FeatureCollection" as const,
      features: [routes[idx][0], routes[idx][1], road],
    };
  }
</script>

<svelte:window on:keydown={onKeyDown} />

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
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>Predict impact</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <Link on:click={() => ($mode = { mode: "predict-impact" })}>
      Pick a different road
    </Link>

    <div style="display: flex; justify-content: space-between;">
      <button disabled={idx == 0} on:click={prev} data-tooltip="Left">
        Previous
      </button>
      {idx + 1} / {routes.length}
      <button
        disabled={idx == routes.length - 1}
        on:click={next}
        data-tooltip="Right"
      >
        Next
      </button>
    </div>

    <p>
      <span style="color: red">Route before</span>
      ,
      <span style="color: blue">route after</span>
    </p>
  </div>

  <div slot="map">
    <GeoJSON data={gj(idx)}>
      <LineLayer
        {...layerId("predict-impact")}
        paint={{
          "line-width": constructMatchExpression(
            ["get", "kind"],
            {
              focus: 8,
            },
            5,
          ),
          "line-color": constructMatchExpression(
            ["get", "kind"],
            {
              focus: "black",
              before: "red",
              after: "blue",
            },
            "cyan",
          ),
          "line-opacity": constructMatchExpression(
            ["get", "kind"],
            {
              focus: 1.0,
            },
            0.5,
          ),
        }}
      />
    </GeoJSON>

    <DotMarker lngLat={gjPosition(routes[idx][0].geometry.coordinates[0])}>
      A
    </DotMarker>
    <DotMarker
      lngLat={gjPosition(
        routes[idx][0].geometry.coordinates[
          routes[idx][0].geometry.coordinates.length - 1
        ],
      )}
    >
      B
    </DotMarker>

    <ModalFilterLayer />
  </div>
</SplitComponent>
