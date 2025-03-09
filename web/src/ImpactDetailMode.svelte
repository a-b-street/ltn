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
  let props = road.properties!;
  props.kind = "focus";

  let routes = $backend!.getImpactsOnRoad(props.id);
  let idx = 0;

  if (routes.length == 0) {
    window.alert(
      "No routes over this road change. (This is a bug in progress of being fixed.)",
    );
    $mode = { mode: "predict-impact" };
  }

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

    <p>
      {props.before.toLocaleString()} routes cross here
      <span style:color="red">before your changes</span>
      , and {props.after.toLocaleString()}
      <span style:color="blue">after your changes</span>
      . That's
      {Math.round((100 * props.after) / props.before)}% of the original traffic.
    </p>

    <PrevNext list={routes} bind:idx />
  </div>

  <div slot="map">
    {#if routes.length > 0}
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
    {/if}

    <ModalFilterLayer />
  </div>
</SplitComponent>
