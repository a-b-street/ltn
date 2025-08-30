<script lang="ts">
  import type {
    DataDrivenPropertyValueSpecification,
    ExpressionSpecification,
  } from "maplibre-gl";
  import { LineLayer, Popup, VectorTileSource } from "svelte-maplibre";
  import { makeRamp } from "svelte-utils/map";
  import { ContextLayerButton, layerId } from "../common";
  import { assetUrl } from "../stores";

  let show = $state(false);
  let purpose = $state("all");
  let scenario = $state("bicycle_go_dutch");
  let networkType = $state("fastest");
  let colorBy = $state("flow");

  let purposes = [
    ["all", "All"],
    ["commute", "Commute"],
    ["primary", "Primary School"],
    ["secondary", "Secondary"],
    ["utility", "Other everyday"],
  ];
  let scenarios = [
    ["bicycle", "Baseline"],
    ["bicycle_go_dutch", "Go Dutch"],
    ["bicycle_ebike", "E-bike"],
  ];
  let networkTypes = [
    ["fastest", "Fast/Direct"],
    ["quietest", "Quiet/Indirect"],
  ];
  let colorByOptions = [
    ["none", "None"],
    ["flow", "People cycling per day"],
    ["quietness", "Cycle friendliness"],
    ["gradient", "Gradient"],
  ];

  let gradient = {
    colorScale: [
      "#59ee19",
      "#37a009",
      "#FFC300",
      "#C70039",
      "#581845",
      "#000000",
    ],
    limits: [3, 5, 7, 10, 100],
  };

  // Implements the formula y = (3 / (1 + exp(-3 * (x / 1000 - 1.6))) + 0.3)
  function lineWidthForDemand(
    input: DataDrivenPropertyValueSpecification<number>,
  ): ExpressionSpecification {
    return [
      "let",
      "base",
      [
        "+",
        0.3,
        ["/", 3, ["+", 1, ["^", 2.718, ["-", 2.94, ["*", input, 0.0021]]]]],
      ],
      [
        "interpolate",
        ["linear"],
        ["zoom"],
        12,
        ["*", 2.1, ["var", "base"]],
        14,
        ["*", 5.25, ["var", "base"]],
        15,
        ["*", 7.5, ["var", "base"]],
        16,
        ["*", 18, ["var", "base"]],
        18,
        ["*", 52.5, ["var", "base"]],
      ],
    ] as ExpressionSpecification;
  }

  function lineColorForDemand(
    input: DataDrivenPropertyValueSpecification<number>,
  ): ExpressionSpecification {
    return [
      "step",
      input,
      "rgba(0,0,0,0)",
      1,
      "#9C9C9C",
      50,
      "#FFFF73",
      100,
      "#AFFF00",
      250,
      "#00FFFF",
      500,
      "#30B0FF",
      1000,
      "#2E5FFF",
      2000,
      "#0000FF",
      3000,
      "#FF00C5",
    ] as ExpressionSpecification;
  }
  let key = $derived(`${purpose}_${networkType}_${scenario}`);
  let lineColor = $derived(
    {
      none: "#304ce7",
      flow: lineColorForDemand(["get", key]),
      quietness: [
        "step",
        ["get", "quietness"],
        "#882255",
        25,
        "#CC6677",
        50,
        "#44AA99",
        75,
        "#117733",
        101,
        "#000000",
      ],
      gradient: makeRamp(
        ["abs", ["get", "gradient"]],
        gradient.limits,
        gradient.colorScale,
      ),
    }[colorBy] as ExpressionSpecification,
  );
</script>

<ContextLayerButton bind:show label="Estimated cycling demand">
  {#snippet help()}
    <p>
      <a href="https://nptscot.github.io/manual/#routenetwork" target="_blank">
        Data from NPT
      </a>
    </p>
  {/snippet}

  {#snippet legend()}
    <label>
      Trip purpose:
      <select bind:value={purpose}>
        {#each purposes as [value, label]}
          <option {value}>{label}</option>
        {/each}
      </select>
    </label>

    <label>
      Scenario:
      <select bind:value={scenario}>
        {#each scenarios as [value, label]}
          <option {value}>{label}</option>
        {/each}
      </select>
    </label>

    <label>
      Network type:
      <select bind:value={networkType}>
        {#each networkTypes as [value, label]}
          <option {value}>{label}</option>
        {/each}
      </select>
    </label>

    <label>
      Color by:
      <select bind:value={colorBy}>
        {#each colorByOptions as [value, label]}
          <option {value}>{label}</option>
        {/each}
      </select>
    </label>
  {/snippet}
</ContextLayerButton>

<VectorTileSource
  url={`pmtiles://${assetUrl("cnt/layers/route_network.pmtiles")}`}
>
  <LineLayer
    {...layerId("context-route-network")}
    sourceLayer="rnet"
    paint={{
      "line-color": lineColor,
      "line-width": lineWidthForDemand(["get", key]),
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    hoverCursor="pointer"
  >
    <Popup openOn="click">
      {#snippet children({ data })}
        {@const props = data!.properties!}
        <div style="max-width: 30vw; max-height: 60vh; overflow: auto;">
          <p>Cyclists: {props[key].toLocaleString()}</p>
          <p>Gradient: {props.gradient}%</p>
          <p>Cycle-friendliness: {props.quietness}%</p>

          <details>
            <summary>All network details</summary>

            <p>Fast/Direct network</p>
            <table>
              <tbody>
                <tr>
                  <td></td>
                  <th>Baseline</th>
                  <th>Go Dutch</th>
                  <th>E-bikes</th>
                </tr>
                {#each purposes as [value, label]}
                  <tr>
                    <th>{label}</th>
                    <td>{props[`${value}_fastest_bicycle`].toLocaleString()}</td
                    >
                    <td>
                      {props[
                        `${value}_fastest_bicycle_go_dutch`
                      ].toLocaleString()}
                    </td>
                    <td>
                      {props[`${value}_fastest_bicycle_ebike`].toLocaleString()}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>

            <p>Quiet/Indirect network</p>
            <table>
              <tbody>
                <tr>
                  <td></td>
                  <th>Baseline</th>
                  <th>Go Dutch</th>
                  <th>E-bikes</th>
                </tr>
                {#each purposes as [value, label]}
                  <tr>
                    <th>{label}</th>
                    <td
                      >{props[`${value}_quietest_bicycle`].toLocaleString()}</td
                    >
                    <td>
                      {props[
                        `${value}_quietest_bicycle_go_dutch`
                      ].toLocaleString()}
                    </td>
                    <td>
                      {props[
                        `${value}_quietest_bicycle_ebike`
                      ].toLocaleString()}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </details>
        </div>
      {/snippet}
    </Popup>
  </LineLayer>
</VectorTileSource>
