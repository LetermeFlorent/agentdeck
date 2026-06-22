/**
 * Test des commandes slash pour Claude Code et opencode.
 * Valide quelles commandes fonctionnent réellement en mode headless/one-shot.
 * Usage : node scripts/test-slash-commands.mjs [--opencode] [--verbose]
 *
 * Même pattern que test-hermes-e2e.mjs (pas de build Tauri requis).
 */
import { spawn } from "node:child_process";
import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { homedir } from "node:os";
import { join, basename, extname } from "node:path";

const HOME = homedir();
const ARGS = process.argv.slice(2);
const TEST_OPENCODE = ARGS.includes("--opencode");
const VERBOSE = ARGS.includes("--verbose");

// ── Utilitaires ──────────────────────────────────────────────────────────────

function log(msg) { console.log(msg); }
function dbg(msg) { if (VERBOSE) console.log("  [dbg]", msg); }

const OK  = (name, snippet) => ({ name, works: true,  snippet: snippet?.slice(0, 80) ?? "" });
const KO  = (name, reason)  => ({ name, works: false, snippet: reason?.slice(0, 80) ?? "" });

// ── 1. Token Claude ───────────────────────────────────────────────────────────
const credsPath = join(HOME, ".claude", ".credentials.json");
let claudeToken = null;
try {
  const raw = readFileSync(credsPath, "utf8");
  claudeToken = JSON.parse(raw)?.claudeAiOauth?.accessToken ?? null;
} catch {
  console.error("❌ Token Claude introuvable :", credsPath);
  process.exit(1);
}
if (!claudeToken) { console.error("❌ Token Claude vide"); process.exit(1); }
log(`✅ Token Claude OK (${claudeToken.length} chars)`);

// ── 2. Scan disque : skills + commands installés ──────────────────────────────

function scanDir(dir, depth, names) {
  if (depth === 0 || !existsSync(dir)) return;
  let entries;
  try { entries = readdirSync(dir); } catch { return; }
  for (const e of entries) {
    const p = join(dir, e);
    let st;
    try { st = statSync(p); } catch { continue; }
    if (st.isDirectory()) {
      // Check SKILL.md inside
      const skillMd = join(p, "SKILL.md");
      if (existsSync(skillMd)) names.add(e); // folder name = command name
      scanDir(p, depth - 1, names);
    } else if (extname(e) === ".md" && basename(dir) === "commands") {
      names.add(basename(e, ".md")); // filename without .md
    }
  }
}

const base = join(HOME, ".claude");
const diskNames = new Set();
scanDir(join(base, "skills"),   4, diskNames);
scanDir(join(base, "commands"), 4, diskNames);
scanDir(join(base, "plugins"),  8, diskNames);

log(`\n📁 Commandes trouvées sur disque : ${diskNames.size}`);
if (diskNames.size > 0) log("   " + [...diskNames].join(", "));

// ── 3. Init event Claude : liste des built-ins ────────────────────────────────

async function getClaudeInitCommands() {
  return new Promise((resolve) => {
    const child = spawn("claude", [
      "-p", "--input-format", "stream-json", "--output-format", "stream-json",
      "--verbose", "--permission-mode", "bypassPermissions", "--model", "haiku",
    ], {
      env: { ...process.env, CLAUDE_CODE_OAUTH_TOKEN: claudeToken, ANTHROPIC_API_KEY: undefined },
      stdio: ["pipe", "pipe", "pipe"],
    });

    const trigger = JSON.stringify({
      type: "user",
      message: { role: "user", content: [{ type: "text", text: "." }] },
    }) + "\n";
    child.stdin.write(trigger);
    child.stdin.end();

    let buf = "";
    let found = [];
    child.stdout.on("data", (chunk) => {
      buf += chunk.toString();
      const lines = buf.split("\n");
      buf = lines.pop();
      for (const line of lines) {
        if (!line.trim()) continue;
        try {
          const v = JSON.parse(line);
          if (v.type === "system" && v.subtype === "init" && Array.isArray(v.slash_commands)) {
            found = v.slash_commands;
            child.kill();
          }
        } catch {}
      }
    });
    child.stderr.on("data", () => {});
    const timer = setTimeout(() => { child.kill(); resolve(found); }, 20000);
    child.on("close", () => { clearTimeout(timer); resolve(found); });
  });
}

log("\n⏳ Récupération des built-ins Claude via init event...");
const initBuiltins = await getClaudeInitCommands();
log(`✅ Init event : ${initBuiltins.length} commandes built-in`);
dbg(initBuiltins.join(", "));

// ── 4. Liste complète à tester ────────────────────────────────────────────────

// Built-ins connus qui NE marchent PAS en mode headless (interface seulement)
const NON_HEADLESS = new Set([
  "clear", "compact", "config", "context", "init", "usage", "usage-credits", "extra-usage",
  "insights", "heapdump", "reload-skills", "goal", "team-onboarding", "model", "agents", "mcp",
  "login", "logout", "status", "doctor", "cost", "resume", "vim", "exit", "quit", "bug", "ide",
  "hooks", "permissions", "output-style", "memory", "add-dir", "release-notes", "privacy-settings",
  "terminal-setup", "pr-comments", "export", "feedback", "upgrade", "statusline", "todos",
]);

// Union : disk + init
const allClaude = new Set([...diskNames, ...initBuiltins]);
// Séparer présumés-ok et présumés-ko
const candidatesOk = [...allClaude].filter(n => !NON_HEADLESS.has(n));
const candidatesKo = [...allClaude].filter(n => NON_HEADLESS.has(n));

log(`\n📋 Commandes à tester :`);
log(`   Présumées fonctionnelles (skills + custom) : ${candidatesOk.length}`);
log(`   Présumées non-headless (built-ins UI)      : ${candidatesKo.length}`);

// ── 5. Tester une commande Claude ─────────────────────────────────────────────

async function testClaudeCmd(cmdName, timeoutMs = 15000) {
  return new Promise((resolve) => {
    const msg = JSON.stringify({
      type: "user",
      message: { role: "user", content: [{ type: "text", text: `/${cmdName}` }] },
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
    let resultText = "";
    let hasError = false;
    let assistantText = "";

    child.stdout.on("data", (chunk) => {
      buf += chunk.toString();
      const lines = buf.split("\n");
      buf = lines.pop();
      for (const line of lines) {
        if (!line.trim()) continue;
        try {
          const v = JSON.parse(line);
          if (v.type === "result") {
            resultText = v.result ?? "";
            if (v.subtype === "error") hasError = true;
          }
          if (v.type === "assistant" && v.message?.content) {
            for (const c of v.message.content) {
              if (c.type === "text") assistantText += c.text;
            }
          }
        } catch {}
      }
    });
    child.stderr.on("data", () => {});

    const timer = setTimeout(() => {
      child.kill();
      resolve(KO(cmdName, "timeout"));
    }, timeoutMs);

    child.on("close", (code) => {
      clearTimeout(timer);
      const combined = (resultText + assistantText).toLowerCase();
      if (hasError || combined.includes("unknown command") || combined.includes("not supported in")) {
        resolve(KO(cmdName, resultText || assistantText));
      } else if (code !== 0 && !resultText && !assistantText) {
        resolve(KO(cmdName, `exit code ${code}`));
      } else {
        // Heuristique : si la réponse est longue et semble de l'IA (>200 chars), c'est du texte AI
        // (commande non interceptée). Si courte ou vide → probablement interceptée.
        const totalLen = (resultText + assistantText).length;
        if (totalLen > 300 && assistantText.length > 200) {
          resolve(OK(cmdName, `[IA répond ~${totalLen}c] ${(resultText || assistantText).slice(0, 60)}`));
        } else {
          resolve(OK(cmdName, resultText || assistantText || "(vide — commande interceptée)"));
        }
      }
    });
  });
}

// ── 6. Tester une commande opencode ──────────────────────────────────────────

function resolveOpencodeBin() {
  const appData = process.env.APPDATA;
  if (appData) {
    const npmBin = join(appData, "npm", "node_modules", "opencode-ai", "bin", "opencode.exe");
    if (existsSync(npmBin)) return npmBin;
  }
  return "opencode";
}

async function testOpencodeCmd(cmdName, timeoutMs = 30000) {
  return new Promise((resolve) => {
    const bin = resolveOpencodeBin();
    const child = spawn(bin, [
      "run", "--format", "json", "--dangerously-skip-permissions",
      "-m", "openrouter/nex-agi/nex-n2-pro:free",
      `/${cmdName}`,
    ], {
      stdio: ["pipe", "pipe", "pipe"],
    });
    child.stdin.end();

    let buf = "";
    let hasError = false;
    let hasText = false;
    let snippet = "";

    child.stdout.on("data", (chunk) => {
      buf += chunk.toString();
      const lines = buf.split("\n");
      buf = lines.pop();
      for (const line of lines) {
        if (!line.trim()) continue;
        try {
          const v = JSON.parse(line);
          if (v.type === "error" || v.type === "session.error") {
            hasError = true;
            snippet = v.error?.message ?? JSON.stringify(v).slice(0, 80);
          }
          if (v.type === "text" && v.text) {
            hasText = true;
            snippet = snippet || v.text.slice(0, 80);
          }
        } catch {}
      }
    });
    child.stderr.on("data", (d) => {
      const s = d.toString().toLowerCase();
      if (s.includes("unknown") || s.includes("error")) {
        hasError = true;
        snippet = snippet || d.toString().slice(0, 80);
      }
    });

    const timer = setTimeout(() => {
      child.kill();
      resolve(KO(cmdName, "timeout"));
    }, timeoutMs);

    child.on("close", (code) => {
      clearTimeout(timer);
      if (hasError || (code !== 0 && !hasText)) {
        resolve(KO(cmdName, snippet || `exit ${code}`));
      } else {
        resolve(OK(cmdName, snippet || "(pas d'erreur)"));
      }
    });
  });
}

// ── 7. Lancer les tests Claude ────────────────────────────────────────────────

log("\n═══════════════════════════════════════════════════");
log("  TEST CLAUDE — commandes présumées fonctionnelles");
log("═══════════════════════════════════════════════════");

const claudeResults = [];

// Skills et commandes custom (disk) + built-ins hors NON_HEADLESS
for (const cmd of candidatesOk) {
  process.stdout.write(`  /${cmd.padEnd(30)} `);
  const r = await testClaudeCmd(cmd);
  claudeResults.push(r);
  console.log(r.works ? `✅  ${r.snippet}` : `❌  ${r.snippet}`);
}

log("\n═══════════════════════════════════════════════════");
log("  TEST CLAUDE — built-ins présumés non-headless");
log("  (validation rapide, on s'attend à KO)");
log("═══════════════════════════════════════════════════");

// Test d'un échantillon représentatif (pas tous, pour aller vite)
const SAMPLE_NON_HEADLESS = ["clear", "compact", "model", "config", "memory", "cost"];
for (const cmd of SAMPLE_NON_HEADLESS.filter(c => allClaude.has(c))) {
  process.stdout.write(`  /${cmd.padEnd(30)} `);
  const r = await testClaudeCmd(cmd, 8000);
  claudeResults.push(r);
  console.log(r.works ? `✅ (surprise!)  ${r.snippet}` : `❌  ${r.snippet}`);
}

// ── 8. Tester opencode ────────────────────────────────────────────────────────

if (TEST_OPENCODE) {
  log("\n═══════════════════════════════════════════════════");
  log("  TEST OPENCODE — commandes officielles");
  log(`  Binaire : ${resolveOpencodeBin()}`);
  log("  Modèle  : openrouter/nex-agi/nex-n2-pro:free");
  log("═══════════════════════════════════════════════════");

  const OPENCODE_CANDIDATES = [
    "new", "compact", "models", "sessions", "share", "unshare",
    "export", "undo", "redo", "connect", "themes", "editor",
    "init", "details", "thinking", "help", "exit",
  ];

  const opencodeResults = [];
  for (const cmd of OPENCODE_CANDIDATES) {
    process.stdout.write(`  /${cmd.padEnd(30)} `);
    const r = await testOpencodeCmd(cmd);
    opencodeResults.push(r);
    console.log(r.works ? `✅  ${r.snippet}` : `❌  ${r.snippet}`);
  }

  log("\n── Résumé opencode ──────────────────────────────────");
  const ocOk = opencodeResults.filter(r => r.works).map(r => r.name);
  const ocKo = opencodeResults.filter(r => !r.works).map(r => r.name);
  log(`  ✅ Fonctionnelles (${ocOk.length}) : ${ocOk.join(", ") || "—"}`);
  log(`  ❌ Cassées (${ocKo.length})        : ${ocKo.join(", ") || "—"}`);
} else {
  log("\n💡 Ajouter --opencode pour tester opencode aussi (nécessite opencode installé).");
}

// ── 9. Résumé Claude ─────────────────────────────────────────────────────────

log("\n═══════════════════════════════════════════════════");
log("  RÉSUMÉ CLAUDE");
log("═══════════════════════════════════════════════════");
const cOk = claudeResults.filter(r => r.works).map(r => r.name);
const cKo = claudeResults.filter(r => !r.works).map(r => r.name);
log(`✅ Fonctionnelles (${cOk.length}) : ${cOk.join(", ") || "—"}`);
log(`❌ Cassées (${cKo.length})        : ${cKo.join(", ") || "—"}`);
log(`\n💡 Commandes disk (skills/custom) ignorées par les tests ci-dessus (toujours OK si elles existent) :`);
log(`   ${[...diskNames].join(", ") || "aucune"}`);
