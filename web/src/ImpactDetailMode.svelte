<script lang="ts">
  import type { Feature, FeatureCollection } from "geojson";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { DotMarker, gjPosition, layerId, Link, PrevNext } from "./common";
  import { ModalFilterLayer } from "./layers";
  import { backend, mode, returnToChooseProject } from "./stores";

  export let road: Feature;

  // TODO Weird to modify the input like this?
  road.properties!.kind = "focus";

  let routes = $backend!.getImpactsOnRoad(road.properties!.id);
  let idx = 0;

  function gj(idx: number): FeatureCollection {
    return {
      type: "FeatureCollection" as const,
      features: [routes[idx][0], routes[idx][1], road],
    };
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={returnToChooseProject}>Choose project</Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "pick-neighbourhood" })}>
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

    <PrevNext list={routes} bind:idx />

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
