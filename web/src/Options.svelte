<script lang="ts">
  export let options: any;
  export let onChange: () => void;
</script>

<details open>
  <summary>Config</summary>

  <label>
    Angle tolerance (TODO describe -- LineStrings pointing in opposite
    directions are ignored?)

    <input
      type="number"
      bind:value={options.angle_tolerance}
      min="0"
      max="90"
      step="1"
      on:change={onChange}
    />
  </label>

  <label>
    Distance tolerance (TODO describe)

    <input
      type="number"
      bind:value={options.distance_tolerance}
      min="1"
      max="50"
      step="1"
      on:change={onChange}
    />
  </label>
</details>
