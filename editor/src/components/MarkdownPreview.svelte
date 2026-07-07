<script lang="ts">
    import { unified } from "unified";
    import remarkParse from "remark-parse";
    import remarkRehype from "remark-rehype";
    import rehypeStringify from "rehype-stringify";
    
    interface Props {
        content: string;
    }
    let { content }: Props = $props();

    let html = $derived(
        unified()
            .use(remarkParse)
            .use(remarkRehype, { allowDangerousHtml: true })
            .use(rehypeStringify, { allowDangerousHtml: true })
            .processSync(content || "*Empty document*")
            .toString()
    );
</script>

<div id="markdown-preview-container" class="markdown-preview p-6 w-full h-full overflow-y-auto bg-[var(--bg-primary)] prose prose-invert max-w-none">
    {@html html}
</div>

<style>
    /* Add basic styling since we dropped Tailwind typography for now */
    :global(.markdown-preview h1) { font-size: 2em; font-weight: bold; margin-bottom: 0.5em; margin-top: 1em; }
    :global(.markdown-preview h2) { font-size: 1.5em; font-weight: bold; margin-bottom: 0.5em; margin-top: 1em; border-bottom: 1px solid var(--border); padding-bottom: 0.3em; }
    :global(.markdown-preview p) { margin-bottom: 1em; line-height: 1.6; }
    :global(.markdown-preview ul) { list-style-type: disc; padding-left: 2em; margin-bottom: 1em; }
    :global(.markdown-preview blockquote) { border-left: 4px solid var(--border); padding-left: 1em; color: var(--text-muted); margin-left: 0; }
</style>
