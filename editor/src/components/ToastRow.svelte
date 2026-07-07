<script lang="ts">
    import { onMount } from "svelte";
    import type { ToastItem } from "../state/toast.svelte.ts";
    import iconCheckBadge from "../assets/mascot/icon-check-badge.png";

    interface Props {
        toast: ToastItem;
        onDismiss: (id: number) => void;
    }

    let { toast, onDismiss }: Props = $props();

    let shown = $state(false);

    onMount(() => {
        const enter = requestAnimationFrame(() => shown = true);
        const fade = window.setTimeout(() => shown = false, toast.duration);
        const remove = window.setTimeout(() => onDismiss(toast.id), toast.duration + 200);

        return () => {
            cancelAnimationFrame(enter);
            clearTimeout(fade);
            clearTimeout(remove);
        };
    });
</script>

<div
    role={toast.type === 'error' ? 'alert' : 'status'}
    aria-live={toast.type === 'error' ? 'assertive' : 'polite'}
    aria-atomic="true"
    class={`px-4 py-2 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-subtle)]
        shadow-lg text-[var(--text-primary)] text-sm font-medium
        transition-all duration-200 ease-out
        ${shown ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2'}`}
>
    <div class="flex items-center gap-2">
        {#if toast.type === 'success'}
            <img src={iconCheckBadge} alt="" aria-hidden="true" draggable="false" class="w-5 h-5 object-contain select-none" />
        {:else if toast.type === 'error'}
            <svg class="w-4 h-4 text-[var(--danger)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
        {:else}
            <svg class="w-4 h-4 text-[var(--accent)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        {/if}
        {toast.message}
    </div>
</div>
