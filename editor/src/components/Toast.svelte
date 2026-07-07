<script lang="ts">
    import { toastState } from "../state/toast.svelte.ts";

    // Individual toast row
    import ToastRow from "./ToastRow.svelte";
</script>

{#if toastState.toasts.length > 0}
    <div class="fixed bottom-20 left-1/2 -translate-x-1/2 z-50 flex flex-col items-center gap-2 pointer-events-none">
        {#each toastState.toasts as toast (toast.id)}
            <ToastRow {toast} onDismiss={toastState.dismiss} />
        {/each}
    </div>
{/if}
