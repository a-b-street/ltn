<script lang="ts">
  import type { Feature, FeatureCollection } from "geojson";
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
        roadGj: Feature;
        gj: FeatureCollection;
        shortcutIndex: number;
      };
  let state: State = { state: "neutral" };

  function choseRoad(roadGj: Feature) {
    let gj = JSON.parse($app!.getShortcutsCrossingRoad(roadGj.properties!.id));
    if (gj.features.length == 0) {
      window.alert("No shortcuts here");
      return;
    }

    state = {
      state: "chose-road",
      roadGj,
      gj,
      shortcutIndex: 0,
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
      if (e.key == "ArrowLeft" && state.shortcutIndex > 0) {
        e.stopPropagation();
        state.shortcutIndex--;
      }
      if (e.key == "ArrowRight") {
        e.stopPropagation();
        if (state.shortcutIndex != state.gj.features.length - 1) {
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
      state.shortcutIndex--;
    }
  }

  function next() {
    if (state.state == "chose-road") {
      state.shortcutIndex++;
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
        <button on:click={() => (state = { state: "neutral" })}
          >Pick a different road</button
        >
      </div>
      <div>
        <button disabled={state.shortcutIndex == 0} on:click={prev}>
          Prev
        </button>
        {state.shortcutIndex + 1} / {state.gj.features.length}
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
        onClickLine={choseRoad}
      >
        <div slot="line-popup">
          <Popup openOn="hover" let:props>
            <p>
              {props.shortcuts} shortcuts through {props.name ?? "unnamed road"}
            </p>
          </Popup>
        </div>
      </RenderNeighbourhood>
    {:else if state.state == "chose-road"}
      <GeoJSON data={state.gj.features[state.shortcutIndex]}>
        <LineLayer
          paint={{
            "line-width": 5,
            "line-color": "red",
          }}
        />
      </GeoJSON>
      <GeoJSON data={state.roadGj}>
        <LineLayer
          paint={{
            "line-width": 5,
            "line-color": "blue",
          }}
        />
      </GeoJSON>
    {/if}
    <ModalFilterLayer />
  </div>
</SplitComponent>
