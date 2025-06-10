<script lang="ts">
  import type { Map } from "maplibre-gl";

  export let map: Map;
  export let finish: (roads: number[]) => void;
  export let cancel: () => void;
</script>
