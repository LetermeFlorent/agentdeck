/**
 * Vérifie une commande slash en la lançant réellement et affichant la sortie complète.
 * Usage : node scripts/verify-slash-cmd.mjs <provider> <commande>
 * Exemples :
 *   node scripts/verify-slash-cmd.mjs claude code-review
 *   node scripts/verify-slash-cmd.mjs claude clear
 *   node scripts/verify-slash-cmd.mjs opencode new
 *
 * Même pattern que test-hermes-e2e.mjs (pas de build Tauri requis).
 */
import { spawn } from "node:child_process";
import { readFileSync, existsSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const HOME = homedir();
const [,, provider = "claude", cmdName = "help"] = process.argv;
const TIMEOUT_MS = 120_000; // 2 min — suffisant pour les skills lents

console.log(`\n🔍 Test /${cmdName} (provider: ${provider})\n`);

// ── Token Claude ──────────────────────────────────────────────────────────────
const credsPath = join(HOME, ".claude", ".credentials.json");
let claudeToken = null;
try {
  claudeToken = JSON.parse(readFileSync(credsPath, "utf8"))?.claudeAiOauth?.accessToken ?? null;
} catch {}

// ── Résolution binaire ────────────────────────────────────────────────────────
function resolveOpencodeBin() {
  const appData = process.env.APPDATA;
  if (appData) {
    const p = join(appData, "npm", "node_modules", "opencode-ai", "bin", "opencode.exe");
    if (existsSync(p)) return p;
  }
  return "opencode";
}

// ── Test Claude ───────────────────────────────────────────────────────────────
async function testClaude(cmd) {
  if (!claudeToken) { console.error("❌ Token Claude manquant"); process.exit(1); }
  const msg = JSON.stringify({
    type: "user",
    message: { role: "user", content: [{ type: "text", text: `/${cmd}` }] },
  }) + "\n";

  const child = spawn("claude", [
    "-p", "--input-format", "stream-json", "--output-format", "stream-json",
    "--verbose", "--permission-mode", "bypassPermissions", "--model", "haiku",
  ], {
    env: { ...process.env, CLAUDE_CODE_OAUTH_TOKEN: claudeToken, ANTHROPIC_API_KEY: undefined },
    stdio: ["pipe", "pipe", "pipe"],
  });

  child.stdin.write(msg);
  child.stdin.end();

  let buf = "";
  const events = [];

  child.stdout.on("data", (chunk) => {
    buf += chunk.toString();
    const lines = buf.split("\n");
    buf = lines.pop();
    for (const line of lines) {
      if (!line.trim()) continue;
      try {
        const v = JSON.parse(line);
        events.push(v);
      } catch {}
    }
  });

  child.stderr.on("data", (d) => {
    const t = d.toString().trim();
    if (t) console.log("  [stderr]", t.slice(0, 120));
  });

  const timer = setTimeout(() => {
    child.kill();
    console.log("⏱  Timeout (2 min)");
  }, TIMEOUT_MS);

  const exitCode = await new Promise((r) => child.on("close", r));
  clearTimeout(timer);

  console.log(`  Exit code : ${exitCode}`);
  console.log(`  Events reçus : ${events.length}\n`);

  // Afficher les events significatifs
  for (const v of events) {
    const t = v.type;
    if (t === "result") {
      const label = v.subtype === "error" ? "❌ RESULT ERROR" : "✅ RESULT";
      console.log(`${label} :`);
      console.log(v.result ?? "(vide)");
    } else if (t === "assistant" && v.message?.content) {
      for (const c of v.message.content) {
        if (c.type === "text" && c.text?.trim()) {
          console.log("💬 ASSISTANT :");
          console.log(c.text.slice(0, 2000));
        }
      }
    } else if (t === "system" && v.subtype === "init") {
      console.log(`📋 INIT : ${v.slash_commands?.length ?? 0} commandes`);
    } else if (["tool_use", "tool_result"].includes(t)) {
      console.log(`🔧 ${t.toUpperCase()} : ${v.name ?? v.tool_use_id ?? ""}`);
    }
  }
}

// ── Test opencode ─────────────────────────────────────────────────────────────
async function testOpencode(cmd) {
  const bin = resolveOpencodeBin();
  console.log(`  Binaire : ${bin}`);

  const child = spawn(bin, [
    "run", "--format", "json", "--dangerously-skip-permissions",
    "-m", "openrouter/nex-agi/nex-n2-pro:free",
    `/${cmd}`,
  ], { stdio: ["pipe", "pipe", "pipe"] });

  child.stdin.end();

  let buf = "";
  const events = [];
  const stderrLines = [];

  child.stdout.on("data", (chunk) => {
    buf += chunk.toString();
    const lines = buf.split("\n");
    buf = lines.pop();
    for (const line of lines) {
      if (!line.trim()) continue;
      try { events.push(JSON.parse(line)); } catch {}
    }
  });
  child.stderr.on("data", (d) => stderrLines.push(d.toString()));

  const timer = setTimeout(() => { child.kill(); console.log("⏱  Timeout"); }, TIMEOUT_MS);
  const exitCode = await new Promise((r) => child.on("close", r));
  clearTimeout(timer);

  console.log(`  Exit code : ${exitCode}`);
  if (stderrLines.length) console.log("  Stderr :", stderrLines.join("").slice(0, 200));
  console.log(`  Events reçus : ${events.length}\n`);

  for (const v of events) {
    if (v.type === "error" || v.type === "session.error") {
      console.log("❌ ERROR :", v.error?.message ?? JSON.stringify(v));
    } else if (v.type === "text") {
      console.log("💬 TEXT :", v.text?.slice(0, 1000));
    } else {
      console.log(`  [${v.type}]`);
    }
  }
}

// ── Main ──────────────────────────────────────────────────────────────────────
if (provider === "opencode") {
  await testOpencode(cmdName);
} else {
  await testClaude(cmdName);
}
