<script lang="ts">
  import type { Feature, FeatureCollection } from "geojson";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
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
      features: [routes[idx][0], routes[idx][1], road].filter((f) => f != null),
    };
  }

  function startPos(idx: number): [number, number] {
    if (routes[idx][0] != null) {
      return gjPosition(routes[idx][0].geometry.coordinates[0]);
    }
    return gjPosition(routes[idx][1]!.geometry.coordinates[0]);
  }

  function endPos(idx: number): [number, number] {
    let pts =
      routes[idx][0] != null
        ? routes[idx][0].geometry.coordinates
        : routes[idx][1]!.geometry.coordinates;
    return gjPosition(pts[pts.length - 1]);
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
    <BackButton on:click={() => ($mode = { mode: "predict-impact" })} />

    <p>
      {props.before.toLocaleString()} routes cross here
      <span style:color="red">before your changes</span>
      , and {props.after.toLocaleString()}
      <span style:color="blue">after your changes</span>
      . That's
      {Math.round((100 * props.after) / props.before)}% of the original traffic.
    </p>

    <p>
      Note: The routes are currently sampled, to speed things up. This one
      sample route may represent many trips between the same points.
    </p>

    <PrevNext list={routes} bind:idx />

    {#if routes[idx][0] == null}
      <p style:color="red">
        No possible route before changes (
        <i>This is usually a known software bug</i>
      </p>
    {/if}
    {#if routes[idx][1] == null}
      <p style:color="blue">
        No possible route after changes (
        <i>This is usually a known software bug</i>
      </p>
    {/if}
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

      <DotMarker lngLat={startPos(idx)}>A</DotMarker>
      <DotMarker lngLat={endPos(idx)}>B</DotMarker>
    {/if}

    <ModalFilterLayer />
  </div>
</SplitComponent>
