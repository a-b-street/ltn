<script lang="ts">
  import type { Feature } from "geojson";
  import type { LngLat } from "maplibre-gl";
  import { GeoJSON, LineLayer, Popup } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import {
    DotMarker,
    gjPosition,
    layerId,
    ModeLink,
    pageTitle,
    PrevNext,
  } from "./common";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    ModalFilterLayer,
    NeighbourhoodRoadLayer,
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
    if (state.state != "neutral") {
      return;
    }

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
</script>

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
        <li>
          <ModeLink mode={{ mode: "neighbourhood" }} />
        </li>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  {/snippet}

  {#snippet left()}
    <BackButton mode={{ mode: "neighbourhood" }} />

    {#if state.state == "neutral"}
      <p>Click a road to see shortcuts</p>
    {:else if state.state == "chose-road"}
      <p>
        This shows all possible shortcuts crossing the blue road you've chosen.
        A shortcut is defined as a route starting and ending on main (busy)
        roads, then cutting through smaller streets. It might not actually be
        considered a "good shortcut" in practice -- this tool doesn't know any
        real traffic patterns; it's just looking for any possible path. This
        view lets you understand the limits of this assumption.
      </p>

      <button onclick={() => (state = { state: "neutral" })}>
        Pick a different road
      </button>

      <PrevNext list={state.gj.features} bind:idx={state.shortcutIndex} />

      <p>
        This shortcut is <b>
          {state.gj.features[
            state.shortcutIndex
          ].properties!.directness.toFixed(1)}x
        </b>
        the length of the shortest route using all roads, not just this neighbourhood
      </p>
    {/if}
  {/snippet}

  {#snippet main()}
    <RenderNeighbourhood>
      <HighlightBoundaryLayer />
      <CellLayer />
      <OneWayLayer />

      <NeighbourhoodRoadLayer
        onClickLine={choseRoad}
        interactive={state.state == "neutral"}
      >
        {#snippet linePopup()}
          {#if state.state == "neutral"}
            <Popup openOn="hover">
              {#snippet children({ data })}
                {@const props = data!.properties!}
                {#if props.kind == "interior_road"}
                  <p>
                    {props.shortcuts} shortcuts through {props.name ??
                      "unnamed road"}
                  </p>
                {:else if props.kind == "main_road"}
                  <p>
                    Main road: {props.name ?? "unnamed road"}
                  </p>
                {/if}
              {/snippet}
            </Popup>
          {/if}
        {/snippet}
      </NeighbourhoodRoadLayer>
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

    <ModalFilterLayer interactive={false} />
  {/snippet}
</SplitComponent>
