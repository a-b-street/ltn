<script lang="ts">
  import { Modal } from "./common";

  export let filterType: string;

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
</script>

<Modal on:close let:dialog>
  <h3>Choose a modal filter to place on streets</h3>
  <table>
    <tr>
      <td>
        {#each choices as [name, label, description]}
          <div>
            <button
              style="width: 100%"
              disabled={filterType == name}
              on:click={() => (filterType = name)}
              ><img src={`/filters/${name}.svg`} />{label}</button
            >
          </div>
        {/each}
      </td>
      <td>
        <img src={`/filters/${filterType}.gif`} height="300" />
        <p>{choices.find((x) => x[0] == filterType)[2]}</p>
      </td>
    </tr>
  </table>
  <center><button on:click={() => dialog.close()}>Confirm</button></center>
</Modal>
