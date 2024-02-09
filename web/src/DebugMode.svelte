<script lang="ts">
  import BackButton from "./BackButton.svelte";
  import { CircleLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { layerId, notNull, PropertiesTable, Popup } from "./common";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "title" })}>
            Choose study area
          </a>
        </li>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </a>
        </li>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "neighbourhood" })}>
            Editing modal filters
          </a>
        </li>
        <li>Debug mode</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "neighbourhood" })} />
  </div>

  <div slot="map">
    <RenderNeighbourhood
      gjInput={JSON.parse(notNull($app).renderNeighbourhood())}
      interactive
      onClickLine={(f) => window.open(notNull(f.properties).way, "_blank")}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </div>
      <svelte:fragment slot="more-layers">
        <CircleLayer
          {...layerId("debug-borders")}
          filter={["==", ["get", "kind"], "border_intersection"]}
          paint={{
            "circle-radius": 15,
            "circle-color": "green",
          }}
        >
          <Popup openOn="hover" let:props>
            <PropertiesTable properties={props} />
          </Popup>
        </CircleLayer>
        <LineLayer
          {...layerId("debug-crosses")}
          filter={["==", ["get", "kind"], "crosses"]}
          paint={{
            "line-width": 5,
            "line-color": "blue",
          }}
        >
          <Popup openOn="hover" let:props>
            <PropertiesTable properties={props} />
          </Popup>
        </LineLayer>
      </svelte:fragment>
    </RenderNeighbourhood>

    <GeoJSON data={JSON.parse(notNull($app).renderModalFilters())} generateId>
      <CircleLayer
        {...layerId("debug-filters")}
        paint={{
          "circle-radius": 15,
          "circle-color": "black",
        }}
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </CircleLayer>
    </GeoJSON>
  </div>
</SplitComponent>
