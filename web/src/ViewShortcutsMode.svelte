<script lang="ts">
  import { LTN } from "backend";
  import type { Map } from "maplibre-gl";
  import { onDestroy, onMount } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import SplitComponent from "./SplitComponent.svelte";

  export let mode: Mode;
  export let app: LTN;
  export let prevMode: Mode;
  export let map: Map;

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
  let state = { state: "neutral" };

  function choseRoad(app: LTN, road: number) {
    let gj = JSON.parse(app.getShortcutsCrossingRoad(road));
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
    map.keyboard.disable();
  });
  onDestroy(() => {
    map.keyboard.enable();
  });

  function onKeyDown(e: KeyboardEvent) {
    if (state.state == "chose-road") {
      if (e.key == "ArrowLeft" && state.shortcutIndex != 0) {
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
    }
  }

  function back() {
    mode = prevMode;
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
          on:click={() => state.shortcutIndex--}
        >
          Prev
        </button>
        {state.shortcutIndex} / {state.gj.features.length}
        <button
          disabled={state.shortcutIndex == state.gj.features.length - 1}
          on:click={() => state.shortcutIndex++}
        >
          Next
        </button>
      </div>
    {/if}
  </div>

  <div slot="map">
    {#if state.state == "neutral"}
      <GeoJSON data={JSON.parse(app.render())}>
        <LineLayer
          paint={{
            "line-width": 5,
            "line-color": "black",
          }}
          on:click={(e) => choseRoad(app, e.detail.features[0].properties.id)}
          hoverCursor="pointer"
        />
      </GeoJSON>
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
  </div>
</SplitComponent>
