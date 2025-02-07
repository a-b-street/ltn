<script lang="ts">
  import { Control } from "svelte-maplibre";
  import CBD from "./CBD.svelte";
  import POIs from "./POIs.svelte";
  import RouteNetwork from "./RouteNetwork.svelte";
</script>

<Control>
  <div class="pico" style="display: flex; flex-direction: column">
    <POIs />
    <CBD />
    <RouteNetwork />
  </div>
</Control>
