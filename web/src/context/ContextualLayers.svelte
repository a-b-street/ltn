<script lang="ts">
  import { Control } from "svelte-maplibre";
  import icon from "../../assets/layers.svg?url";
  import CBD from "./CBD.svelte";
  import POIs from "./POIs.svelte";
  import RouteNetwork from "./RouteNetwork.svelte";

  let expand = false;
</script>

<Control defaultStyling={true}>
  <div
    class="pico"
    style:background="grey"
    style="display: flex; flex-direction: column"
  >
    <button on:click={() => (expand = !expand)}>
      <img src={icon} title="Layers" alt="Layers" />
    </button>

    <div
      style="display: flex; flex-direction: column"
      style:visibility={expand ? "visible" : "collapse"}
    >
      <POIs />
      <CBD />
      <RouteNetwork />
    </div>
  </div>
</Control>
