<script lang="ts">
  import BackButton from "./BackButton.svelte";
  import { setCellColors } from "./cells";
  import type { Feature, FeatureCollection } from "geojson";
  import { onDestroy, onMount } from "svelte";
  import { FillLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { layerId, notNull, Popup, Link } from "./common";
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
        <li>
          <Link on:click={back}>Editing modal filters</Link>
        </li>
        <li>Viewing shortcuts</li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <BackButton on:click={back} />

    {#if state.state == "neutral"}
      <p>Click a road to see shortcuts</p>
    {:else if state.state == "chose-road"}
      <p>
        This shows all possible shortcuts crossing the blue road you've chosen.
        A shortcut is defined as a route starting and ending outside the
        neighbourhood, but cutting through it. It might not actually be
        considered a "good shortcut" in practice -- this tool doesn't know any
        real traffic patterns; it's just looking for any possible path. This
        view lets you understand the limits of this assumption.
      </p>
      <button on:click={() => (state = { state: "neutral" })}>
        Pick a different road
      </button>
      <div style="display: flex; justify-content: space-between;">
        <button disabled={state.shortcutIndex == 0} on:click={prev}>
          Previous
        </button>
        {state.shortcutIndex + 1} / {state.gj.features.length}
        <button
          disabled={state.shortcutIndex == state.gj.features.length - 1}
          on:click={next}
        >
          Next
        </button>
      </div>
      <p>
        This shortcut is <b>
          {notNull(
            state.gj.features[state.shortcutIndex].properties,
          ).directness.toFixed(1)}x
        </b>
        the length of the shortest route using all roads, not just this neighbourhood
      </p>
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
      <GeoJSON
        data={setCellColors(JSON.parse(notNull($app).renderNeighbourhood()))}
      >
        <FillLayer
          {...layerId("cells")}
          filter={["==", ["get", "kind"], "cell"]}
          paint={{
            "fill-color": ["get", "color"],
            "fill-opacity": 0.3,
          }}
        />
      </GeoJSON>

      <GeoJSON data={state.gj.features[state.shortcutIndex]}>
        <LineLayer
          {...layerId("shortcuts")}
          paint={{
            "line-width": 5,
            "line-color": "red",
          }}
        />
      </GeoJSON>
      <GeoJSON data={state.roadGj}>
        <LineLayer
          {...layerId("shortcuts-focus")}
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
