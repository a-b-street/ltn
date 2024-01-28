<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import { onDestroy, onMount } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { notNull, Popup } from "./common";
  import ModalFilterLayer from "./ModalFilterLayer.svelte";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, map, mode } from "./stores";

  type State =
    | {
        state: "neutral";
      }
    | {
        state: "chose-road";
        road: number;
        gj: FeatureCollection;
        shortcutIndex: number | null;
      };
  let state: State = { state: "neutral" };

  function choseRoad(road: number) {
    let gj = JSON.parse($app!.getShortcutsCrossingRoad(road));
    if (gj.features.length == 0) {
      window.alert("No shortcuts here");
      return;
    }

    state = {
      state: "chose-road",
      road,
      gj,
      shortcutIndex: null,
    };
  }

  onMount(() => {
    $map?.keyboard.disable();
  });
  onDestroy(() => {
    $map?.keyboard.enable();
  });

  function onKeyDown(e: KeyboardEvent) {
    if (state.state == "chose-road") {
      if (e.key == "ArrowLeft" && state.shortcutIndex) {
        e.stopPropagation();
        state.shortcutIndex--;
      }
      if (e.key == "ArrowRight") {
        e.stopPropagation();
        if (state.shortcutIndex == null) {
          state.shortcutIndex = 0;
        } else if (state.shortcutIndex != state.gj.features.length - 1) {
          state.shortcutIndex++;
        }
      }
      if (e.key == "Escape") {
        e.stopPropagation();
        back();
      }
    }
  }

  function back() {
    $mode = { mode: "neighbourhood" };
  }

  function prev() {
    if (state.state == "chose-road") {
      state.shortcutIndex!--;
    }
  }

  function next() {
    if (state.state == "chose-road") {
      state.shortcutIndex =
        state.shortcutIndex == null ? 0 : state.shortcutIndex + 1;
    }
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="sidebar">
    <div><button on:click={back}>Back to editing</button></div>

    {#if state.state == "neutral"}
      <p>Click a road to see shortcuts</p>
    {:else if state.state == "chose-road"}
      <div>
        <button
          disabled={state.shortcutIndex == null || state.shortcutIndex == 0}
          on:click={prev}
        >
          Prev
        </button>
        {state.shortcutIndex} / {state.gj.features.length}
        <button
          disabled={state.shortcutIndex == state.gj.features.length - 1}
          on:click={next}
        >
          Next
        </button>
      </div>
    {/if}
  </div>

  <div slot="map">
    {#if state.state == "neutral"}
      <RenderNeighbourhood
        gjInput={JSON.parse(notNull($app).renderNeighbourhood())}
        onClickLine={(f) => choseRoad(notNull(f.properties).id)}
      >
        <div slot="line-popup">
          <Popup openOn="hover" let:props>{props.shortcuts}</Popup>
        </div>
      </RenderNeighbourhood>
    {:else if state.state == "chose-road"}
      {#if state.shortcutIndex == null}
        <GeoJSON data={state.gj}>
          <LineLayer
            paint={{
              "line-width": 5,
              "line-color": "red",
            }}
          />
        </GeoJSON>
      {:else}
        <GeoJSON data={state.gj.features[state.shortcutIndex]}>
          <LineLayer
            paint={{
              "line-width": 5,
              "line-color": "red",
            }}
          />
        </GeoJSON>
      {/if}
    {/if}
    <ModalFilterLayer />
  </div>
</SplitComponent>
