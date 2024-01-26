<script lang="ts">
  import { CircleLayer, Popup } from "svelte-maplibre";
  import { notNull, PropertiesTable } from "./common";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";
</script>

<SplitComponent>
  <div slot="sidebar">
    <h1>Debug mode</h1>

    <div>
      <button on:click={() => ($mode = { mode: "neighbourhood" })}
        >Back to editing</button
      >
    </div>
  </div>

  <div slot="map">
    <RenderNeighbourhood
      gjInput={JSON.parse(notNull($app).renderNeighbourhood())}
      interactive
      onClickLine={(f) => window.open(notNull(f.properties).way, "_blank")}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={notNull(data).properties} />
        </Popup>
      </div>
      <div slot="circle-popup">
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={notNull(data).properties} />
        </Popup>
      </div>
      <svelte:fragment slot="more-layers">
        <CircleLayer
          filter={["==", ["get", "kind"], "border_intersection"]}
          paint={{
            "circle-radius": 15,
            "circle-color": "green",
          }}
        >
          <Popup openOn="hover" let:data>
            <PropertiesTable properties={notNull(data).properties} />
          </Popup>
        </CircleLayer>
      </svelte:fragment>
    </RenderNeighbourhood>
  </div>
</SplitComponent>
