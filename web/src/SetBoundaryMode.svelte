<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import type { AreaProps } from "route-snapper-ts";
  import { notNull } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { gjPosition, ModeLink, pageTitle } from "./common";
  import AreaControls from "./common/draw_area/AreaControls.svelte";
  import { type Waypoint } from "./common/draw_area/stores";
  import { backend, map, mode, saveCurrentProject } from "./stores";

  export let name: string;
  export let existing: Feature<Polygon, AreaProps>;
  let waypoints: Waypoint[] = [];
  let drawnShape: Feature<Polygon>;

  if (existing.properties.waypoints) {
    // Transform into the correct format
    waypoints = existing.properties.waypoints.map((waypt) => {
      return {
        point: [waypt.lon, waypt.lat],
        snapped: waypt.snapped,
      };
    });
  } else {
    // No stored waypoints -- this is either a boundary drawn with a very old
    // version of this tool, or an auto-generated boundary. Just
    // "backfill" by using the full geometry as freehand points.
    // Editing will be very painful in practice, but it won't break.
    // Note the second polygon ring is used, because the boundary is expressed as
    // "everywhere" minus a hole for the boundary, to achieve the fade-outside effect.
    waypoints = existing.geometry.coordinates[1].slice(1).map((point) => {
      return { point: gjPosition(point), snapped: false };
    });
  }

  function finish() {
    if (drawnShape) {
      try {
        $backend!.setCurrentNeighbourhoodBoundary(name, drawnShape);
        saveCurrentProject();
        $mode = { mode: "neighbourhood" };
      } catch (err) {
        window.alert(`Sorry, this boundary is invalid: ${err}`);
        cancel();
      }
    }
  }

  function cancel() {
    $mode = {
      mode: "neighbourhood",
    };
  }
</script>

<SplitComponent>
  <div slot="top">
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
  </div>
  <div slot="sidebar">
    <BackButton mode={{ mode: "neighbourhood" }} />

    <h1>Adjust boundary</h1>

    <AreaControls
      map={notNull($map)}
      bind:waypoints
      bind:drawnShapeOut={drawnShape}
    />

    <div style="display: flex; gap: 16px;">
      <button on:click={finish} disabled={waypoints.length < 3}>Finish</button>
      <button class="secondary" on:click={cancel}>Cancel</button>
    </div>
  </div>
</SplitComponent>
