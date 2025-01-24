<script lang="ts">
  import type { Feature } from "geojson";
  import type { LngLat } from "maplibre-gl";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { DotMarker, gjPosition, layerId, Link, PrevNext } from "./common";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    InteriorRoadLayer,
    ModalFilterLayer,
    OneWayLayer,
    RenderNeighbourhood,
  } from "./layers";
  import { backend, mode } from "./stores";
  import type { AllShortcuts } from "./wasm";

  type State =
    | {
        state: "neutral";
      }
    | {
        state: "chose-road";
        roadGj: Feature;
        gj: AllShortcuts;
        shortcutIndex: number;
      };
  let state: State = { state: "neutral" };

  function choseRoad(roadGj: Feature, _: LngLat) {
    let gj = $backend!.getShortcutsCrossingRoad(roadGj.properties!.id);
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

  function back() {
    $mode = { mode: "neighbourhood" };
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "pick-neighbourhood" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>
          <Link on:click={back}>Editing</Link>
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

      <PrevNext list={state.gj.features} bind:idx={state.shortcutIndex} />

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
    <RenderNeighbourhood>
      <HighlightBoundaryLayer />
      <CellLayer />
      <OneWayLayer />

      {#if state.state == "neutral"}
        <InteriorRoadLayer onClickLine={choseRoad}>
          <div slot="line-popup">
            <Popup openOn="hover" let:props>
              <p>
                {props.shortcuts} shortcuts through {props.name ??
                  "unnamed road"}
              </p>
            </Popup>
          </div>
        </InteriorRoadLayer>
      {:else if state.state == "chose-road"}
        <InteriorRoadLayer interactive={false} />
      {/if}
    </RenderNeighbourhood>

    {#if state.state == "chose-road"}
      <GeoJSON data={state.gj.features[state.shortcutIndex]}>
        <LineLayer
          {...layerId("shortcuts")}
          paint={{
            "line-width": 10,
            "line-color": "cyan",
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

      <DotMarker
        lngLat={gjPosition(
          state.gj.features[state.shortcutIndex].geometry.coordinates[0],
        )}
      >
        A
      </DotMarker>
      <DotMarker
        lngLat={gjPosition(
          state.gj.features[state.shortcutIndex].geometry.coordinates[
            state.gj.features[state.shortcutIndex].geometry.coordinates.length -
              1
          ],
        )}
      >
        B
      </DotMarker>
    {/if}

    <ModalFilterLayer />
  </div>
</SplitComponent>
