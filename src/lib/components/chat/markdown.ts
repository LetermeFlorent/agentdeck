// Mini-rendu Markdown sûr pour le chat : on échappe TOUT le HTML d'abord, puis on
// applique un sous-ensemble (titres, gras, italique, code, listes, citations, liens http).
// Aucune balise issue du texte n'est interprétée → pas d'injection.

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/** Transforme le balisage inline (sur du texte déjà échappé). */
function inline(s: string): string {
  // code `…`
  s = s.replace(/`([^`]+)`/g, (_m, c) => `<code>${c}</code>`);
  // gras **…** / __…__
  s = s.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  s = s.replace(/__([^_]+)__/g, "<strong>$1</strong>");
  // italique *…* / _…_
  s = s.replace(/(^|[^*])\*([^*\s][^*]*?)\*/g, "$1<em>$2</em>");
  s = s.replace(/(^|[^_])_([^_\s][^_]*?)_/g, "$1<em>$2</em>");
  // liens [texte](http…) — schéma http(s) uniquement
  s = s.replace(
    /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g,
    '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>',
  );
  return s;
}

export function renderMarkdown(src: string): string {
  const lines = src.split("\n");
  let html = "";
  let inCode = false;
  let codeBuf: string[] = [];
  let listOpen = false;
  const closeList = () => {
    if (listOpen) {
      html += "</ul>";
      listOpen = false;
    }
  };
  for (const line of lines) {
    if (/^\s*```/.test(line)) {
      if (!inCode) {
        closeList();
        inCode = true;
        codeBuf = [];
      } else {
        html += `<pre><code>${esc(codeBuf.join("\n"))}</code></pre>`;
        inCode = false;
      }
      continue;
    }
    if (inCode) {
      codeBuf.push(line);
      continue;
    }
    const h = line.match(/^(#{1,6})\s+(.*)$/);
    if (h) {
      closeList();
      const lvl = h[1].length;
      html += `<h${lvl}>${inline(esc(h[2]))}</h${lvl}>`;
      continue;
    }
    const li = line.match(/^\s*[-*]\s+(.*)$/);
    if (li) {
      if (!listOpen) {
        html += "<ul>";
        listOpen = true;
      }
      html += `<li>${inline(esc(li[1]))}</li>`;
      continue;
    }
    const quote = line.match(/^\s*>\s?(.*)$/);
    if (quote) {
      closeList();
      html += `<blockquote>${inline(esc(quote[1]))}</blockquote>`;
      continue;
    }
    if (line.trim() === "") {
      closeList();
      continue;
    }
    closeList();
    html += `<p>${inline(esc(line))}</p>`;
  }
  if (inCode) html += `<pre><code>${esc(codeBuf.join("\n"))}</code></pre>`;
  closeList();
  return html;
}
