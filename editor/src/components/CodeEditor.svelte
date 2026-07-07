<script lang="ts">
    import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, drawSelection, dropCursor } from "@codemirror/view";
    import { EditorState, Compartment } from "@codemirror/state";
    import { history, defaultKeymap, historyKeymap } from "@codemirror/commands";
    import { markdown } from "@codemirror/lang-markdown";
    import { syntaxHighlighting, HighlightStyle } from "@codemirror/language";
    import { tags as t } from "@lezer/highlight";
    import { vim } from "@replit/codemirror-vim";
    import { settingsState } from "../state/settings.svelte.ts";

    interface Props {
        content: string;
        onChange: (content: string) => void;
    }
    let { content, onChange }: Props = $props();

    const markdownHighlight = HighlightStyle.define([
        { tag: t.heading1, color: "var(--syntax-h1)", fontWeight: "bold" },
        { tag: t.heading2, color: "var(--syntax-h2)", fontWeight: "bold" },
        { tag: [t.heading3, t.heading4, t.heading5, t.heading6], color: "var(--syntax-h3)", fontWeight: "600" },
        { tag: t.strong, color: "var(--syntax-bold)", fontWeight: "bold" },
        { tag: t.emphasis, fontStyle: "italic" },
        { tag: t.link, color: "var(--syntax-link)" },
    ]);

    const editorTheme = EditorView.theme({
        "&": { height: "100%", color: "var(--text-primary)", backgroundColor: "var(--bg-editor)", fontSize: "14px", borderRight: "1px solid var(--border)" },
        ".cm-scroller": { overflow: "auto", fontFamily: "'JetBrains Mono', ui-monospace, monospace" },
        ".cm-content": { caretColor: "var(--accent)", padding: "16px 0" },
        ".cm-gutters": { backgroundColor: "var(--bg-gutter)", color: "var(--text-muted)", border: "none", borderRight: "1px solid var(--border-subtle)" }
    });

    const vimCompartment = new Compartment();

    function codemirror(node: HTMLElement, initContent: string) {
        const view = new EditorView({
            state: EditorState.create({
                doc: initContent,
                extensions: [
                    lineNumbers(),
                    highlightActiveLineGutter(),
                    history(),
                    drawSelection(),
                    dropCursor(),
                    editorTheme,
                    markdown(),
                    syntaxHighlighting(markdownHighlight),
                    keymap.of([...defaultKeymap, ...historyKeymap]),
                    vimCompartment.of(settingsState.vimMode ? vim() : []),
                    EditorView.updateListener.of((update) => {
                        if (update.docChanged) {
                            onChange(update.state.doc.toString());
                        }
                    })
                ]
            }),
            parent: node
        });

        // Watch settingsState.vimMode to update the compartment dynamically
        let unsub = $effect.root(() => {
            $effect(() => {
                view.dispatch({
                    effects: vimCompartment.reconfigure(settingsState.vimMode ? vim() : [])
                });
            });
        });

        return {
            update(newContent: string) {
                if (view.state.doc.toString() !== newContent) {
                    view.dispatch({
                        changes: { from: 0, to: view.state.doc.length, insert: newContent }
                    });
                }
            },
            destroy() {
                unsub();
                view.destroy();
            }
        };
    }
</script>

<div class="h-full w-full" use:codemirror={content}></div>
