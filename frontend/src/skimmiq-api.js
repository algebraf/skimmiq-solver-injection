export async function solveSkimmiqNative(state, options = {}) {
  const startedAt = performanceNow();
  const response = await fetch(apiUrl(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      state,
      profile: options.profile || defaultProfileForState(state),
      timeoutMs: options.timeoutMs ?? 300000
    })
  });

  const text = await response.text();
  let payload = null;
  try {
    payload = text ? JSON.parse(text) : null;
  } catch {
    throw new Error(`SkimmIQ solver returned invalid JSON: ${text.slice(0, 240)}`);
  }

  if (!response.ok || !payload) {
    const message = payload?.error || payload?.message || `HTTP ${response.status}`;
    throw new Error(`SkimmIQ solver failed: ${message}`);
  }

  const status = payload.status === "solved"
    ? "solved"
    : payload.status === "timeout"
      ? "timeout"
      : "search_exhausted";

  return {
    status,
    moves: Array.isArray(payload.moves) ? payload.moves : [],
    text: payload.text || "",
    method: payload.method ? `native-${payload.method}` : "native",
    nodes: Number(payload.nodes || 0),
    elapsedMs: Number(payload.elapsedMs || Math.round(performanceNow() - startedAt))
  };
}

function defaultProfileForState(state) {
  const layout = String(state?.layoutId || state?.layout || "").toUpperCase();
  const difficulty = String(state?.difficultyId || state?.difficulty || "").toLowerCase();
  if (layout === "E" && difficulty === "classic") return "quality";
  if (layout === "E" && difficulty !== "classic") return "balanced";
  if (layout === "B" && difficulty === "classic") return "balanced";
  return "fast";
}

function apiUrl() {
  const here = self.location.href;
  const path = self.location.pathname;
  const relative = path.includes("/assets/") ? "../api/solve.php" : "./api/solve.php";
  return new URL(relative, here).toString();
}

function performanceNow() {
  if (typeof performance !== "undefined" && performance.now) return performance.now();
  return Date.now();
}
