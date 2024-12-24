<script lang="ts">
  import { onMount } from "svelte";
  import "@picocss/pico/css/pico.jade.min.css";
  import type { Map } from "maplibre-gl";
  import {
    GeoJSON,
    MapLibre,
    LineLayer,
    hoverStateFilter,
  } from "svelte-maplibre";
  import { Layout } from "svelte-utils/two_column_layout";
  import { emptyGeojson, bbox } from "svelte-utils/map";
  import type { Feature, FeatureCollection } from "geojson";
  import init, { matchLineStrings } from "anime_js";
  import Options from "./Options.svelte";

  let map: Map | undefined;

  let sourceGj = emptyGeojson();
  let sourceColor = "red";

  let targetGj = emptyGeojson();
  let targetColor = "blue";
  let hoveredTarget: Feature | null = null;

  let options = {
    distance_tolerance: 15.0,
    angle_tolerance: 10.0,
  };

  interface MatchCandidate {
    source_index: number;
    shared_length: number;
  }
  let matches: { [idx: number]: MatchCandidate[] } = {};

  onMount(async () => {
    await init();
  });

  $: matchSourceIndices =
    hoveredTarget == null
      ? []
      : (matches[hoveredTarget.id] ?? []).map((x) => x.source_index);

  function recalculate() {
    try {
      matches = JSON.parse(
        matchLineStrings(
          JSON.stringify(sourceGj),
          JSON.stringify(targetGj),
          options,
        ),
      );
    } catch (err) {
      window.alert(`Bug: ${err}`);
      return;
    }

    // Modify targetGj, so we can style based on matches
    for (let [idx, f] of targetGj.features.entries()) {
      f.properties.has_match = matches[idx] != null;
    }
    targetGj = targetGj;
  }

  let fileInput: HTMLInputElement;
  async function loadFiles(e: Event) {
    if (!fileInput.files) {
      return;
    }
    let len = fileInput.files.length;
    if (len != 2) {
      window.alert("Select two GeoJSON files");
      return;
    }

    try {
      sourceGj = await loadFile(fileInput.files[0]);
      targetGj = await loadFile(fileInput.files[1]);
      zoomFit();
      hoveredTarget = null;
      recalculate();
    } catch (err) {
      window.alert(`Bad input file: ${err}`);
    }
  }

  async function loadFile(file: File): Promise<FeatureCollection> {
    let text = await file.text();
    let gj = JSON.parse(text);

    // Overwrite feature IDs
    let id = 0;
    for (let f of gj.features) {
      f.id = id++;
      // Make sure properties aren't null
      f.properties ??= {};
    }

    return gj;
  }

  function zoomFit() {
    let gj = {
      type: "FeatureCollection" as const,
      features: [...sourceGj.features, ...targetGj.features],
    };
    map?.fitBounds(bbox(gj), {
      animate: false,
      padding: 10,
    });
  }

  function swap() {
    [sourceGj, targetGj] = [targetGj, sourceGj];
    recalculate();
  }
</script>

<Layout>
  <div slot="left">
    <h1>Match LineStrings</h1>

    <label>
      Load two .geojson files
      <input bind:this={fileInput} on:change={loadFiles} type="file" multiple />
    </label>

    {#if sourceGj.features.length > 0}
      <hr />

      <div>
        <button on:click={swap}>Swap</button>
      </div>
      <div><button on:click={zoomFit}>Zoom to fit</button></div>

      <div style:background={sourceColor}>Sources</div>
      <p>{sourceGj.features.length} sources</p>

      <div style:background={targetColor}>Target</div>
      <p>
        {targetGj.features.length} targets, with {Object.values(matches).filter(
          (x) => x.length > 0,
        ).length} matching some source
      </p>

      <Options bind:options onChange={recalculate} />
    {/if}
  </div>

  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style="https://api.maptiler.com/maps/dataviz/style.json?key=MZEJTanw3WpxRvt7qDfo"
      standardControls
      bind:map
      on:error={(e) => {
        // @ts-ignore ErrorEvent isn't exported
        console.log(e.detail.error);
      }}
    >
      <GeoJSON data={sourceGj}>
        <LineLayer
          manageHoverState
          paint={{
            "line-width": [
              "case",
              ["in", ["id"], ["literal", matchSourceIndices]],
              8,
              5,
            ],
            "line-color": sourceColor,
            "line-opacity": hoverStateFilter(0.5, 1.0),
          }}
        />
      </GeoJSON>

      <GeoJSON data={targetGj}>
        <LineLayer
          manageHoverState
          paint={{
            "line-width": 8,
            "line-color": targetColor,
            "line-opacity": hoverStateFilter(
              ["case", ["get", "has_match"], 0.5, 0.2],
              1.0,
            ),
          }}
          bind:hovered={hoveredTarget}
        />
      </GeoJSON>
    </MapLibre>
  </div>
</Layout>
