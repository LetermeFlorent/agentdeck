// Action de tooltip stylée (remplace le `title` natif moche).
// Usage : <button use:tooltip={"Mon texte"}>…</button>

export function tooltip(node: HTMLElement, text: string) {
  let tip: HTMLDivElement | null = null;
  let current = text;

  function place() {
    if (!tip) return;
    const r = node.getBoundingClientRect();
    const tr = tip.getBoundingClientRect();
    let top = r.top - tr.height - 8;
    let below = false;
    if (top < 6) {
      top = r.bottom + 8;
      below = true;
    }
    let left = r.left + r.width / 2 - tr.width / 2;
    left = Math.max(6, Math.min(left, window.innerWidth - tr.width - 6));
    tip.style.top = `${top}px`;
    tip.style.left = `${left}px`;
    tip.classList.toggle("below", below);
  }

  function show() {
    if (!current || tip) return;
    tip = document.createElement("div");
    tip.className = "tip";
    tip.textContent = current;
    document.body.appendChild(tip);
    place();
    requestAnimationFrame(() => tip && tip.classList.add("tip-in"));
  }

  function hide() {
    if (tip) {
      tip.remove();
      tip = null;
    }
  }

  node.addEventListener("mouseenter", show);
  node.addEventListener("mouseleave", hide);
  node.addEventListener("pointerdown", hide);

  return {
    update(t: string) {
      current = t;
      if (tip) tip.textContent = t;
    },
    destroy() {
      hide();
      node.removeEventListener("mouseenter", show);
      node.removeEventListener("mouseleave", hide);
      node.removeEventListener("pointerdown", hide);
    },
  };
}
