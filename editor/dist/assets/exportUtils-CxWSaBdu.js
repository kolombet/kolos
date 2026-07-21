import{i as e,n as t,r as n,t as r}from"./index-BEigUco7.js";var i;(function(e){e[e.Audio=1]=`Audio`,e[e.Cache=2]=`Cache`,e[e.Config=3]=`Config`,e[e.Data=4]=`Data`,e[e.LocalData=5]=`LocalData`,e[e.Document=6]=`Document`,e[e.Download=7]=`Download`,e[e.Picture=8]=`Picture`,e[e.Public=9]=`Public`,e[e.Video=10]=`Video`,e[e.Resource=11]=`Resource`,e[e.Temp=12]=`Temp`,e[e.AppConfig=13]=`AppConfig`,e[e.AppData=14]=`AppData`,e[e.AppLocalData=15]=`AppLocalData`,e[e.AppCache=16]=`AppCache`,e[e.AppLog=17]=`AppLog`,e[e.Desktop=18]=`Desktop`,e[e.Executable=19]=`Executable`,e[e.Font=20]=`Font`,e[e.Home=21]=`Home`,e[e.Runtime=22]=`Runtime`,e[e.Template=23]=`Template`})(i||={});var a;(function(e){e[e.Start=0]=`Start`,e[e.Current=1]=`Current`,e[e.End=2]=`End`})(a||={});function o(e){return{isFile:e.isFile,isDirectory:e.isDirectory,isSymlink:e.isSymlink,size:e.size,mtime:e.mtime===null?null:new Date(e.mtime),atime:e.atime===null?null:new Date(e.atime),birthtime:e.birthtime===null?null:new Date(e.birthtime),readonly:e.readonly,fileAttributes:e.fileAttributes,dev:e.dev,ino:e.ino,mode:e.mode,nlink:e.nlink,uid:e.uid,gid:e.gid,rdev:e.rdev,blksize:e.blksize,blocks:e.blocks}}function s(e){let t=new Uint8ClampedArray(e),n=t.byteLength,r=0;for(let e=0;e<n;e++){let n=t[e];r*=256,r+=n}return r}var c=class extends t{async read(e){if(e.byteLength===0)return 0;let t=await n(`plugin:fs|read`,{rid:this.rid,len:e.byteLength}),r=s(t.slice(-8)),i=t instanceof ArrayBuffer?new Uint8Array(t):t;return e.set(i.slice(0,i.length-8)),r===0?null:r}async seek(e,t){return await n(`plugin:fs|seek`,{rid:this.rid,offset:e,whence:t})}async stat(){return o(await n(`plugin:fs|fstat`,{rid:this.rid}))}async truncate(e){await n(`plugin:fs|ftruncate`,{rid:this.rid,len:e})}async write(e){return await n(`plugin:fs|write`,{rid:this.rid,data:e})}};async function l(e,t){if(e instanceof URL&&e.protocol!==`file:`)throw TypeError(`Must be a file URL.`);return new c(await n(`plugin:fs|open`,{path:e instanceof URL?e.toString():e,options:t}))}async function u(e,t,r){if(e instanceof URL&&e.protocol!==`file:`)throw TypeError(`Must be a file URL.`);if(t instanceof ReadableStream){let n=await l(e,{read:!1,create:!0,write:!0,...r}),i=t.getReader();try{for(;;){let{done:e,value:t}=await i.read();if(e)break;await n.write(t)}}finally{i.releaseLock(),await n.close()}}else await n(`plugin:fs|write_file`,t,{headers:{path:encodeURIComponent(e instanceof URL?e.toString():e),options:JSON.stringify(r)}})}async function d(e,t,r){if(e instanceof URL&&e.protocol!==`file:`)throw TypeError(`Must be a file URL.`);await n(`plugin:fs|write_text_file`,new TextEncoder().encode(t),{headers:{path:encodeURIComponent(e instanceof URL?e.toString():e),options:JSON.stringify(r)}})}var f={dark:{bgPrimary:`#0a0a0a`,bgSecondary:`#141414`,textPrimary:`#ffffff`,textSecondary:`#737373`,border:`#262626`,codeBg:`#141414`,codeText:`#a3a3a3`,blockquoteBg:`rgba(20, 20, 20, 0.8)`,accent:`#ffffff`,syntaxH1:`#ffffff`,syntaxH2:`#e5e5e5`,syntaxH3:`#d4d4d4`,syntaxLink:`#a3a3a3`,syntaxBold:`#ffffff`},light:{bgPrimary:`#ffffff`,bgSecondary:`#fafafa`,textPrimary:`#171717`,textSecondary:`#525252`,border:`#e5e5e5`,codeBg:`#f5f5f5`,codeText:`#dc2626`,blockquoteBg:`rgba(250, 250, 250, 0.8)`,accent:`#171717`,syntaxH1:`#171717`,syntaxH2:`#262626`,syntaxH3:`#404040`,syntaxLink:`#2563eb`,syntaxBold:`#171717`},paper:{bgPrimary:`#f5f0e6`,bgSecondary:`#ebe5d8`,textPrimary:`#3d3d3d`,textSecondary:`#6b6352`,border:`#d4cfc2`,codeBg:`#ebe5d8`,codeText:`#8b5a2b`,blockquoteBg:`rgba(235, 229, 216, 0.6)`,accent:`#5c4033`,syntaxH1:`#3d3029`,syntaxH2:`#5c4033`,syntaxH3:`#6b5344`,syntaxLink:`#2d5a7b`,syntaxBold:`#5c4033`},dracula:{bgPrimary:`#282a36`,bgSecondary:`#343746`,textPrimary:`#f8f8f2`,textSecondary:`#d6d6d6`,border:`#44475a`,codeBg:`#21222c`,codeText:`#f8f8f2`,blockquoteBg:`rgba(68, 71, 90, 0.35)`,accent:`#bd93f9`,syntaxH1:`#ff79c6`,syntaxH2:`#ff79c6`,syntaxH3:`#bd93f9`,syntaxLink:`#8be9fd`,syntaxBold:`#f8f8f2`}},p={inter:`'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`,merriweather:`'Merriweather', Georgia, 'Times New Roman', serif`,lora:`'Lora', Georgia, 'Times New Roman', serif`,"source-serif":`'Source Serif 4', Georgia, 'Times New Roman', serif`,"fira-sans":`'Fira Sans', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`},m={small:{base:`14px`,h1:`1.875em`,h2:`1.5em`,h3:`1.125em`,lineHeight:`1.6`},medium:{base:`16px`,h1:`2.25em`,h2:`1.75em`,h3:`1.25em`,lineHeight:`1.7`},large:{base:`18px`,h1:`2.5em`,h2:`2em`,h3:`1.375em`,lineHeight:`1.8`}};function h(e,t,n){let r=f[e],i=p[t],a=m[n];return`
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: ${i};
            font-size: ${a.base};
            line-height: ${a.lineHeight};
            background-color: ${r.bgPrimary};
            color: ${r.textPrimary};
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
            padding: 3rem;
            max-width: 800px;
            margin: 0 auto;
        }

        @page {
            margin: 18mm 16mm;
        }

        @media print {
            html, body {
                background: #ffffff;
            }
            body {
                padding: 0;
                max-width: none;
                color: #171717;
            }
            /* Browsers drop background fills when printing unless asked; keep
               code blocks, table headers and blockquote tints visible. */
            pre, code, th, blockquote, .hljs {
                -webkit-print-color-adjust: exact;
                print-color-adjust: exact;
            }
            /* Keep atomic blocks and their headings from splitting awkwardly. */
            pre, blockquote, table, img, tr {
                page-break-inside: avoid;
                break-inside: avoid;
            }
            h1, h2, h3, h4, h5, h6 {
                page-break-after: avoid;
                break-after: avoid;
            }
        }

        h1 {
            font-size: ${a.h1};
            font-weight: 800;
            padding-bottom: 0.3em;
            border-bottom: 1px solid ${r.border};
            color: ${r.syntaxH1};
            margin-bottom: 1rem;
            margin-top: 0;
        }

        h2 {
            font-size: ${a.h2};
            font-weight: 700;
            padding-bottom: 0.3em;
            border-bottom: 1px solid ${r.border};
            color: ${r.syntaxH2};
            margin-top: 2rem;
            margin-bottom: 1rem;
        }

        h3 {
            font-size: ${a.h3};
            font-weight: 600;
            color: ${r.syntaxH3};
            margin-top: 1.5rem;
            margin-bottom: 0.5rem;
        }

        h4, h5, h6 {
            font-weight: 600;
            color: ${r.syntaxH3};
            margin-top: 1.25rem;
            margin-bottom: 0.5rem;
        }

        p {
            margin-bottom: 1rem;
        }

        a {
            color: ${r.syntaxLink};
            text-decoration: none;
        }

        a:hover {
            text-decoration: underline;
        }

        strong {
            font-weight: 600;
            color: ${r.syntaxBold};
        }

        em {
            font-style: italic;
        }

        code {
            font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
            background: ${r.codeBg};
            border: 1px solid ${r.border};
            border-radius: 0.25rem;
            padding: 0.1em 0.3em;
            font-size: 0.875em;
            color: ${r.codeText};
        }

        pre {
            background: ${r.codeBg};
            border: 1px solid ${r.border};
            border-radius: 0.375rem;
            padding: 1rem;
            overflow-x: auto;
            margin: 1rem 0;
        }

        pre code {
            background: none;
            border: none;
            padding: 0;
            color: ${r.textPrimary};
            font-size: 0.9em;
        }

        ul, ol {
            padding-left: 1.5rem;
            margin-bottom: 1rem;
        }

        li {
            margin-bottom: 0.25rem;
        }

        li > ul, li > ol {
            margin-top: 0.25rem;
            margin-bottom: 0;
        }

        blockquote {
            border-left: 4px solid ${r.accent};
            background: ${r.blockquoteBg};
            padding: 0.5rem 1rem;
            margin: 1rem 0;
            font-style: italic;
            color: ${r.textSecondary};
            border-radius: 0 0.25rem 0.25rem 0;
        }

        blockquote p:last-child {
            margin-bottom: 0;
        }

        hr {
            border: none;
            border-top: 1px solid ${r.border};
            margin: 2rem 0;
        }

        table {
            width: 100%;
            border-collapse: collapse;
            margin: 1rem 0;
        }

        th, td {
            border: 1px solid ${r.border};
            padding: 0.5rem 0.75rem;
            text-align: left;
        }

        th {
            background: ${r.bgSecondary};
            font-weight: 600;
        }

        img {
            max-width: 100%;
            height: auto;
            border-radius: 0.375rem;
            margin: 1rem 0;
        }

        /* Task lists */
        input[type="checkbox"] {
            margin-right: 0.5rem;
            transform: scale(1.1);
        }

        /* Syntax highlighting */
        .hljs-keyword { color: ${r.syntaxH2}; }
        .hljs-string { color: ${r.syntaxBold}; }
        .hljs-number { color: ${r.syntaxH1}; }
        .hljs-function { color: #22c55e; }
        .hljs-comment { color: ${r.textSecondary}; font-style: italic; }
        .hljs-title { color: #22c55e; }
        .hljs-params { color: ${r.textSecondary}; }
        .hljs-built_in { color: ${r.syntaxLink}; }
        .hljs-attr { color: #22c55e; }
        .hljs-literal { color: ${r.syntaxH1}; }

        /* Footer */
        .export-footer {
            margin-top: 3rem;
            padding-top: 1rem;
            border-top: 1px solid ${r.border};
            text-align: center;
            font-size: 0.75rem;
            color: ${r.textSecondary};
        }
    `}async function g(e){let t=new DOMParser().parseFromString(`<div id="__export_root">${e}</div>`,`text/html`),n=t.getElementById(`__export_root`);if(!n)return e;n.querySelectorAll(`button`).forEach(e=>e.remove()),n.querySelectorAll(`.material-symbols-outlined`).forEach(e=>e.remove()),n.querySelectorAll(`a[href^='wikilink:']`).forEach(e=>{let n=t.createElement(`span`);n.textContent=e.textContent,e.replaceWith(n)});for(let e of Array.from(n.querySelectorAll(`img`))){let t=e.getAttribute(`src`)||``;if(t.startsWith(`blob:`))try{let n=await(await fetch(t)).blob(),r=await new Promise((e,t)=>{let r=new FileReader;r.onload=()=>e(r.result),r.onerror=()=>t(r.error),r.readAsDataURL(n)});e.setAttribute(`src`,r)}catch{}}return n.innerHTML}function _(e){return e.replace(/&/g,`&amp;`).replace(/</g,`&lt;`).replace(/>/g,`&gt;`).replace(/"/g,`&quot;`)}function v(e,t,n,r,i,a=!0){let o=h(n,r,i),s=_(t),c=new Date().toLocaleDateString(`en-US`,{year:`numeric`,month:`long`,day:`numeric`}),l=a?`<footer class="export-footer">Exported from Paperling on ${c}</footer>`:``;return`<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="generator" content="Paperling">
    <meta name="date" content="${new Date().toISOString()}">
    <title>${s}</title>
    <style>${o}</style>
</head>
<body>
    <article>
        ${e}
    </article>
    ${l}
</body>
</html>`}async function y(e,t,n,i,a){let o=t.replace(/\.(md|markdown)$/i,``),s=v(await g(e),o,n,i,a),c=await r({defaultPath:`${o}.html`,filters:[{name:`HTML`,extensions:[`html`]}]});return c?(await d(c,s),!0):!1}async function b(t,n,i,a,o){if(!t||t.trim()===``)return console.error(`No HTML content to export!`),!1;let s=n.replace(/\.(md|markdown)$/i,``),c=await g(t),l=`<!DOCTYPE html><html><head><meta charset="utf-8"><title>${_(s)}</title></head><body><article>${c}</article></body></html>`,d=await r({defaultPath:`${s}.docx`,filters:[{name:`Word Document`,extensions:[`docx`]}]});if(!d)return!1;let f=await e(()=>import(`./html-to-docx.browser.esm-CfyH2IyI.js`),[]),p=await(f.default??f)(l,null,{title:s,creator:`Paperling`,footer:!1,pageNumber:!1,font:`Calibri`,fontSize:22,table:{row:{cantSplit:!0}}});return await u(d,p instanceof Blob?new Uint8Array(await p.arrayBuffer()):p instanceof Uint8Array?p:new Uint8Array(p)),!0}var x=`__paperling_print_frame`;function S(e){let t=Array.from(e.images??[]);return Promise.all(t.map(e=>e.complete?Promise.resolve():new Promise(t=>{e.addEventListener(`load`,()=>t(),{once:!0}),e.addEventListener(`error`,()=>t(),{once:!0})}))).then(()=>void 0)}function C(e){return new Promise(t=>{document.getElementById(x)?.remove();let n=document.createElement(`iframe`);n.id=x,n.setAttribute(`aria-hidden`,`true`),n.setAttribute(`tabindex`,`-1`),Object.assign(n.style,{position:`fixed`,left:`-9999px`,top:`0`,width:`794px`,height:`0`,border:`0`,opacity:`0`,pointerEvents:`none`});let r=!1,i=()=>{r||(r=!0,t())};n.onload=()=>{let e=n.contentWindow;if(!e){n.remove(),i();return}let t,r=()=>{e.removeEventListener(`afterprint`,r),clearTimeout(t),setTimeout(()=>n.remove(),300),i()};e.addEventListener(`afterprint`,r),t=setTimeout(r,12e4),Promise.all([e.document.fonts?.ready?.catch(()=>void 0),S(e.document)]).then(()=>{try{e.focus(),e.print()}catch{r()}})},document.body.appendChild(n),n.srcdoc=e})}async function w(e,t,i,a,o){if(!e||e.trim()===``)return console.error(`No HTML content to export!`),`cancelled`;let s=t.replace(/\.(md|markdown)$/i,``),c=v(await g(e),s,`light`,a,o,!0);if(typeof navigator<`u`&&/Windows/i.test(navigator.userAgent)){let e=await r({defaultPath:`${s}.pdf`,filters:[{name:`PDF`,extensions:[`pdf`]}]});return e?(await n(`export_pdf`,{html:c,path:e}),`saved`):`cancelled`}return await C(c),`printing`}export{b as exportToDocx,y as exportToHTML,w as exportToPDF,v as generateHTML,g as prepareExportHtml};