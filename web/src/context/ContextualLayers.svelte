<script lang="ts">
  import { Control } from "svelte-maplibre";
  import icon from "../../assets/layers.svg?url";
  import BusRoutes from "./BusRoutes.svelte";
  import CBD from "./CBD.svelte";
  import POIs from "./POIs.svelte";
  import Population from "./Population.svelte";
  import RailwayStations from "./RailwayStations.svelte";
  import RouteNetwork from "./RouteNetwork.svelte";
  import Stats19 from "./Stats19.svelte";

  let expand = false;
</script>

<Control defaultStyling={true}>
  <div
    class="pico"
    style:background="grey"
    style="display: flex; flex-direction: column; max-height: 80vh; overflow: auto"
  >
    <button on:click={() => (expand = !expand)}>
      <img src={icon} title="Layers" alt="Layers" />
    </button>

    <div
      style="display: flex; flex-direction: column"
      style:visibility={expand ? "visible" : "collapse"}
    >
      <POIs />
      <Population />
      <RailwayStations />
      <BusRoutes />
      <CBD />
      <RouteNetwork />
      <Stats19 />
    </div>
  </div>
</Control>
