<script lang="ts">
  import { Modal } from "svelte-utils";
  import { filterType } from "../stores";

  // TODO Use of import.meta.env.BASE_URL below is to workaround https://github.com/vitejs/vite/issues/10601

  let choices = [
    [
      "walk_cycle_only",
      "Walking/cycling only",
      "A physical barrier that only allows people walking, cycling, and rolling to pass. Often planters or bollards. Larger vehicles cannot enter.",
    ],
    [
      "no_entry",
      "No entry",
      "An alternative sign to indicate vehicles are not allowed to enter the street. Only people walking, cycling, and rolling may pass through.",
    ],
    [
      "bus_gate",
      "Bus gate",
      "A bus gate sign and traffic cameras are installed to allow buses, pedestrians, and cyclists to pass. There is no physical barrier.",
    ],
    [
      "school_street",
      "School street",
      "A closure during school hours only. The barrier usually allows teachers and staff to access the school.",
    ],
  ];

  export let show: boolean;
  $: currentTriple = choices.find((x) => x[0] == $filterType)!;
</script>

<Modal bind:show>
  <h3>Choose a modal filter to place on streets</h3>
  <table>
    <tr>
      <td>
        {#each choices as [name, label, _description]}
          <button
            class="outline"
            style="width: 100%"
            disabled={$filterType == name}
            on:click={() => ($filterType = name)}
          >
            <img
              src={`${import.meta.env.BASE_URL}/filters/${name}_icon.gif`}
              width="80"
              alt={label}
            />
            <br />
            {label}
          </button>
        {/each}
      </td>
      <td>
        <img
          src={`${import.meta.env.BASE_URL}/filters/${$filterType}.gif`}
          height="300"
          alt={currentTriple[1]}
        />
        <p>{currentTriple[2]}</p>
      </td>
    </tr>
  </table>
  <center>
    <button on:click={() => (show = false)}>Confirm</button>
  </center>
</Modal>
