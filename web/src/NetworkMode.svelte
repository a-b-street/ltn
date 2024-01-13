<script lang="ts">
  import { GeoJSON, LineLayer, Popup } from "svelte-maplibre";
  import { notNull, PropertiesTable } from "./common";
  import ManageSavefiles from "./ManageSavefiles.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";

  function resetTitle() {
    $mode = { mode: "title" };
    $app = null;
  }
</script>

<SplitComponent>
  <div slot="sidebar">
    <h1>Define neighbourhood boundaries</h1>
    <p>
      Inside the neighbourhood you define, the goal is to eliminate (or
      deliberately permit) through-traffic. An appropriate neighbourhood
      boundary depends on many factors. The simplest approach is to find the
      area bounded on all sides by "main" roads, which are designed for higher
      traffic volumes. There are many other considerations, though -- maybe
      severances like rivers or rail should be part of a boundary. Bridges and
      tunnels near a boundary may be confusing as well. And note that your
      boundary may not match the conventional definition of "neighbourhood."
    </p>

    <div>
      <button on:click={resetTitle}
        >Start over and change your study area</button
      >
    </div>
    <div>
      <button
        on:click={() => ($mode = { mode: "set-boundary", existing: null })}
        >Draw a new boundary</button
      >
    </div>

    <hr />
    <ManageSavefiles />
  </div>

  <div slot="map">
    <GeoJSON data={JSON.parse(notNull($app).render())}>
      <LineLayer
        id="network"
        paint={{
          "line-width": 5,
          "line-color": "black",
        }}
        on:click={(e) =>
          window.open(notNull(e.detail.features[0].properties).way, "_blank")}
        hoverCursor="pointer"
      >
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={notNull(data).properties} />
        </Popup>
      </LineLayer>
    </GeoJSON>
  </div>
</SplitComponent>
