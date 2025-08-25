<script lang="ts">
  import type { Feature, FeatureCollection } from "geojson";
  import { onMount } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import {
    DotMarker,
    gjPosition,
    layerId,
    Loading,
    ModeLink,
    pageTitle,
    PrevNext,
    refreshLoadingScreen,
  } from "./common";
  import { ModalFilterLayer } from "./layers";
  import { backend, fastSample, mode } from "./stores";
  import type { ImpactOnRoad } from "./wasm";

  export let road: Feature;
  export let prevPrevMode: "pick-neighbourhood" | "neighbourhood";

  // TODO Weird to modify the input like this?
  let data = road.properties!;
  data.kind = "focus";

  let routes: ImpactOnRoad[] = [];
  let idx = 0;

  let loading = "Finding changes to this road";
  onMount(async () => {
    await refreshLoadingScreen();
    routes = $backend!.getImpactsOnRoad(data.id, $fastSample);
    loading = "";
    if (routes.length == 0) {
      window.alert(
        "No routes over this road change. (This is a bug in progress of being fixed.)",
      );
      $mode = { mode: "predict-impact", prevMode: prevPrevMode };
    }
  });

  function gj(idx: number): FeatureCollection {
    return {
      type: "FeatureCollection" as const,
      features: [routes[idx].before, routes[idx].after, road].filter(
        (f) => f != null,
      ),
    };
  }

  function startPos(idx: number): [number, number] {
    if (routes[idx].before != null) {
      return gjPosition(routes[idx].before.geometry.coordinates[0]);
    }
    return gjPosition(routes[idx].after!.geometry.coordinates[0]);
  }

  function endPos(idx: number): [number, number] {
    let pts =
      routes[idx].before != null
        ? routes[idx].before.geometry.coordinates
        : routes[idx].after!.geometry.coordinates;
    return gjPosition(pts[pts.length - 1]);
  }
</script>

<Loading {loading} />

<SplitComponent>
  {#snippet top()}
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <ModeLink mode={{ mode: "title" }} />
        </li>
        <li>
          <ModeLink mode={{ mode: "pick-neighbourhood" }} />
        </li>
        {#if prevPrevMode == "neighbourhood"}
          <li>
            <ModeLink mode={{ mode: "neighbourhood" }} />
          </li>
        {/if}
        <li>
          <ModeLink mode={{ mode: "predict-impact", prevMode: prevPrevMode }} />
        </li>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  {/snippet}

  {#snippet left()}
    <BackButton mode={{ mode: "predict-impact", prevMode: prevPrevMode }} />

    <p>
      {data.before.toLocaleString()} routes cross here
      <span style:color="red">before your changes</span>
      , and {data.after.toLocaleString()}
      <span style:color="blue">after your changes</span>
      . That's
      {Math.round((100 * data.after) / data.before)}% of the original traffic.
    </p>

    {#if routes.length > 0}
      <PrevNext list={routes} bind:idx />

      {#if routes[idx].count > 1}
        <p>
          The routes are currently sampled, to speed things up. This one sample
          route represents {routes[idx].count.toLocaleString()} trips between the
          same points.
        </p>
        <i>
          Note: if these don't sum to the total above, that's likely a known
          software bug
        </i>
      {/if}

      {#if routes[idx].before == null}
        <p style:color="red">
          No possible route before changes (
          <i>This is usually a known software bug</i>
        </p>
      {/if}
      {#if routes[idx].after == null}
        <p style:color="blue">
          No possible route after changes (
          <i>This is usually a known software bug</i>
        </p>
      {/if}
    {/if}
  {/snippet}

  {#snippet main()}
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

    <ModalFilterLayer interactive={false} />
  {/snippet}
</SplitComponent>
