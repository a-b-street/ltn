<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { GeoJSON, LineLayer, Marker } from "svelte-maplibre";
  import { layerId, constructMatchExpression, notNull } from "./common";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, map, mode, route_pt_a, route_pt_b } from "./stores";

  export let prevMode: "network" | "neighbourhood";

  $: gj = JSON.parse(
    $app!.compareRoute(
      $route_pt_a.lng,
      $route_pt_a.lat,
      $route_pt_b.lng,
      $route_pt_b.lat,
    ),
  );

  onMount(() => {
    $map?.keyboard.disable();
  });
  onDestroy(() => {
    $map?.keyboard.enable();
  });

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape") {
      e.stopPropagation();
      back();
    }
  }

  function back() {
    $mode = { mode: prevMode };
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="sidebar">
    <nav aria-label="breadcrumb">
      <!-- svelte-ignore a11y-invalid-attribute -->
      <ul>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "title" })}
            >Change study area</a
          >
        </li>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "network" })}
            >Change neighbourhood</a
          >
        </li>
        {#if prevMode == "neighbourhood"}
          <li>
            <a href="#" on:click={() => ($mode = { mode: "neighbourhood" })}
              >Editing modal filters</a
            >
          </li>
        {/if}
        <li>Routing</li>
      </ul>
    </nav>

    <button on:click={back}>Back</button>

    <p>Drag markers for a route</p>
    <p>
      <span style="color: red">Route before</span>,
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
    <Marker bind:lngLat={$route_pt_a} draggable
      ><span class="dot">A</span></Marker
    >
    <Marker bind:lngLat={$route_pt_b} draggable
      ><span class="dot">B</span></Marker
    >
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
