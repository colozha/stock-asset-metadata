/** Two native pointer/keyboard splitters; layout preferences never contain asset data. */
export function setupPanelResize() {
  const workspace = document.getElementById("workspace")!;
  const handles = [document.getElementById("resize-assets")!, document.getElementById("resize-metadata")!];
  const minimums = [200, 300];
  const storageKey = "stock-asset-metadata.panel-widths";
  let preferred: number[] | null = null;
  let widths = [240, 340];
  try {
    const value: unknown = JSON.parse(localStorage.getItem(storageKey) ?? "null");
    if (Array.isArray(value) && value.length === 2 && value.every((n) => typeof n === "number" && Number.isFinite(n) && n > 0)) preferred = value;
  } catch { /* Unavailable/corrupt storage uses the defaults. */ }
  function maximum(index: number) {
    return Math.max(minimums[index], workspace.clientWidth - 240 - 14 - widths[1-index]);
  }
  function draw() {
    if (window.innerWidth <= 900) return;
    widths = preferred ? [...preferred] : window.innerWidth <= 1180 ? [210, 310] : [240, 340];
    const available = workspace.clientWidth - 240 - 14;
    widths[0] = Math.max(minimums[0], Math.min(widths[0], available - minimums[1]));
    widths[1] = Math.max(minimums[1], Math.min(widths[1], available - widths[0]));
    workspace.style.setProperty("--asset-width", `${widths[0]}px`);
    workspace.style.setProperty("--metadata-width", `${widths[1]}px`);
    handles.forEach((handle,index) => {
      handle.setAttribute("aria-valuemin", String(minimums[index]));
      handle.setAttribute("aria-valuemax", String(Math.floor(maximum(index))));
      handle.setAttribute("aria-valuenow", String(Math.round(widths[index])));
    });
  }
  function save() {
    try {
      if (preferred) localStorage.setItem(storageKey, JSON.stringify(preferred));
      else localStorage.removeItem(storageKey);
    } catch { /* Resizing still works without persistence. */ }
  }
  handles.forEach((handle, index) => {
    let drag: { pointer: number; x: number; width: number } | null = null;
    function setWidth(width: number) {
      widths[index] = Math.max(minimums[index], Math.min(width, maximum(index)));
      preferred = [...widths];
      draw();
    }
    handle.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) return;
      event.preventDefault();
      handle.focus();
      drag = { pointer: event.pointerId, x: event.clientX, width: widths[index] };
      handle.setPointerCapture(event.pointerId);
      handle.classList.add("is-dragging");
      document.documentElement.classList.add("resizing-panels");
    });
    handle.addEventListener("pointermove", (event) => {
      if (drag && event.pointerId === drag.pointer) setWidth(drag.width + (event.clientX-drag.x) * (index === 0 ? 1 : -1));
    });
    function finish() {
      if (!drag) return;
      drag = null;
      handle.classList.remove("is-dragging");
      document.documentElement.classList.remove("resizing-panels");
      save();
    }
    handle.addEventListener("pointerup", finish);
    handle.addEventListener("pointercancel", finish);
    handle.addEventListener("lostpointercapture", finish);
    handle.addEventListener("dblclick", () => { preferred = null; draw(); save(); });
    handle.addEventListener("keydown", (event) => {
      if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
      event.preventDefault();
      const step = (event.shiftKey ? 50 : 10) * (index === 0 ? 1 : -1);
      setWidth(event.key === "Home" ? minimums[index] : event.key === "End" ? maximum(index)
        : widths[index] + (event.key === "ArrowRight" ? step : -step));
      save();
    });
  });
  new ResizeObserver(draw).observe(workspace);
  draw();
}
