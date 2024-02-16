<script lang="ts">
  import BackButton from "./BackButton.svelte";
  import { GeoJSON, LineLayer, Marker } from "svelte-maplibre";
  import { layerId, constructMatchExpression, notNull, Link } from "./common";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode, route_pt_a, route_pt_b } from "./stores";

  export let prevMode: "network" | "neighbourhood";

  $: gj = JSON.parse(
    $app!.compareRoute(
      $route_pt_a.lng,
      $route_pt_a.lat,
      $route_pt_b.lng,
      $route_pt_b.lat,
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
            Choose study area
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
    display: inline-block;
    background-color: grey;
    text-align: center;
  }
</style>
