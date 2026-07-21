<script lang="ts">
  import { onMount } from "svelte";
  import "../app.css";
  import { themeState } from "$lib/state/theme.svelte";

  let { children } = $props();

  onMount(() => {
    return themeState.initListener();
  });
</script>

{@render children()}
