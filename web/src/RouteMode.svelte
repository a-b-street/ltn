<script lang="ts">
  import BackButton from "./BackButton.svelte";
  import { GeoJSON, LineLayer, Marker } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { constructMatchExpression } from "svelte-utils/map";
  import { layerId, Link } from "./common";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { app, mode, route_pt_a, route_pt_b, mainRoadPenalty } from "./stores";

  export let prevMode: "network" | "neighbourhood";

  $: gj = JSON.parse(
    $app!.compareRoute(
      $route_pt_a.lng,
      $route_pt_a.lat,
      $route_pt_b.lng,
      $route_pt_b.lat,
      $mainRoadPenalty,
    ),
  );

  function back() {
    $mode = { mode: prevMode };
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title" })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        {#if prevMode == "neighbourhood"}
          <li>
            <Link on:click={() => ($mode = { mode: "neighbourhood" })}>
              Editing
            </Link>
          </li>
        {/if}
        <li>Routing</li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <BackButton on:click={back} />

    <p>Drag markers for a route</p>
    <p>
      <span style="color: red">Route before</span>
      ,
      <span style="color: blue">route after</span>
    </p>

    <label>
      Slow-down factor for main roads: {$mainRoadPenalty}
      <input
        type="range"
        bind:value={$mainRoadPenalty}
        min="1.0"
        max="5.0"
        step="0.1"
      />
    </label>
    <i>
      Increase to see how drivers may detour in heavy traffic. 1 means
      free-flow.
    </i>
  </div>

  <div slot="map">
    {#if prevMode == "neighbourhood"}
      <RenderNeighbourhood
        gjInput={JSON.parse(notNull($app).renderNeighbourhood())}
        interactive={false}
      />
    {/if}
    <ModalFilterLayer />
    <GeoJSON data={gj}>
      <LineLayer
        {...layerId("compare-route")}
        paint={{
          "line-width": 10,
          "line-color": constructMatchExpression(
            ["get", "kind"],
            {
              before: "red",
              after: "blue",
            },
            "red",
          ),
        }}
      />
    </GeoJSON>
    <Marker bind:lngLat={$route_pt_a} draggable>
      <span class="dot">A</span>
    </Marker>
    <Marker bind:lngLat={$route_pt_b} draggable>
      <span class="dot">B</span>
    </Marker>
  </div>
</SplitComponent>

<style>
  .dot {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: flex;
    justify-content: center;
    align-items: center;

    background-color: grey;
    font-weight: bold;
  }
</style>
