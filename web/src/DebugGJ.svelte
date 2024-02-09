<script lang="ts">
  import BackButton from "./BackButton.svelte";
  import {
    hoverStateFilter,
    CircleLayer,
    GeoJSON,
    LineLayer,
  } from "svelte-maplibre";
  import { Popup, notNull, PropertiesTable } from "./common";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";

  // TODO This should just be a standalone tool
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <!-- svelte-ignore a11y-invalid-attribute -->
      <ul>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "title" })}
            >Choose study area</a
          >
        </li>
        <li>Debug route snapper</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "network" })} />
  </div>

  <div slot="map">
    <GeoJSON data={JSON.parse(notNull($app).toRouteSnapperGj())} generateId>
      <LineLayer
        manageHoverState
        paint={{
          "line-width": 8,
          "line-color": "black",
          "line-opacity": hoverStateFilter(0.5, 1.0),
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </LineLayer>
      <CircleLayer
        manageHoverState
        paint={{
          "circle-radius": 10,
          "circle-color": "black",
          "circle-opacity": hoverStateFilter(0.5, 1.0),
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </CircleLayer>
    </GeoJSON>
  </div>
</SplitComponent>
