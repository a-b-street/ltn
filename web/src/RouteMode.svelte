<script lang="ts">
  import { LngLat } from "maplibre-gl";
  import { onDestroy, onMount } from "svelte";
  import { GeoJSON, LineLayer, Marker } from "svelte-maplibre";
  import { constructMatchExpression, notNull } from "./common";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, map, mode } from "./stores";

  let pt_a: LngLat = randomPoint();
  let pt_b: LngLat = randomPoint();

  $: gj = JSON.parse(
    $app!.compareRoute(pt_a.lng, pt_a.lat, pt_b.lng, pt_b.lat),
  );

  onMount(() => {
    $map?.keyboard.disable();
  });
  onDestroy(() => {
    $map?.keyboard.enable();
  });

  function randomPoint(): LngLat {
    let bounds = $app!.getBounds();
    let lng = bounds[0] + Math.random() * (bounds[2] - bounds[0]);
    let lat = bounds[1] + Math.random() * (bounds[3] - bounds[1]);
    return new LngLat(lng, lat);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape") {
      e.stopPropagation();
      back();
    }
  }

  function back() {
    $mode = { mode: "neighbourhood" };
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="sidebar">
    <div><button on:click={back}>Back to editing</button></div>

    <p>Drag markers for a route</p>
    <p>
      <span style="color: red">Route before</span>,
      <span style="color: blue">route after</span>
    </p>
  </div>

  <div slot="map">
    <RenderNeighbourhood
      gjInput={JSON.parse(notNull($app).renderNeighbourhood())}
      interactive={false}
    />
    <ModalFilterLayer />
    <GeoJSON data={gj}>
      <LineLayer
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
    <Marker bind:lngLat={pt_a} draggable><span class="dot">A</span></Marker>
    <Marker bind:lngLat={pt_b} draggable><span class="dot">B</span></Marker>
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
