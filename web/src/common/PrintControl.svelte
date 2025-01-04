<script lang="ts">
  import {
    DPI,
    Format,
    MaplibreExportControl,
    PageOrientation,
    Size,
  } from "@watergis/maplibre-gl-export";
  import type { Map } from "maplibre-gl";
  import { onDestroy } from "svelte";

  export let map: Map | null;
  export let position: maplibregl.ControlPosition = "top-right";

  let control: MaplibreExportControl | null = null;
  $: if (map && !control) {
    control = new MaplibreExportControl({
      PageSize: Size.A3,
      PageOrientation: PageOrientation.Portrait,
      Format: Format.PNG,
      DPI: DPI[96],
      Crosshair: true,
      PrintableArea: true,
      Local: "en",
    });
    map.addControl(control, position);
  }

  onDestroy(() => {
    if (map?.loaded() && control) {
      map.removeControl(control);
    }
  });
</script>
