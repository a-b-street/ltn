<script lang="ts">
  import { Languages } from "lucide-svelte";
  import { locale } from "../stores";

  // Don't change the store value
  let value = $state(JSON.parse(JSON.stringify($locale)));

  function onchange() {
    const url = new URL(window.location.href);
    if (value == "en") {
      url.searchParams.delete("lang");
    } else {
      url.searchParams.set("lang", value);
    }
    // @ts-expect-error This does work
    window.location.href = url;
  }
</script>

<label>
  <span style="font-size: 20px; margin-left: 8px; margin-top: 4px;">
    <Languages /> Language
  </span>
  <select bind:value {onchange} style="width: 80%">
    <option value="en">English</option>
    <option value="fr">Français</option>
    <option value="hu">Magyar</option>
  </select>
</label>
