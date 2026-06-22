/**
 * Test E2E du flow Hermes (reflect_and_learn) sans Tauri.
 * Réplique exactement ce que fait src-tauri/src/commands/learn.rs.
 * Usage : node scripts/test-hermes-e2e.mjs
 */
import { spawn } from "node:child_process";
import { readFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const HOME = homedir();
const SKILLS_DIR = join(HOME, ".claude", "skills");

// ── 1. Lire le token OAuth ────────────────────────────────────────────────────
const credsPath = join(HOME, ".claude", ".credentials.json");
let token = null;
try {
  const raw = readFileSync(credsPath, "utf8");
  const j = JSON.parse(raw);
  token = j?.claudeAiOauth?.accessToken ?? null;
} catch {
  console.error("❌ Impossible de lire le token :", credsPath);
  process.exit(1);
}
if (!token) { console.error("❌ Token vide"); process.exit(1); }
console.log("✅ Token OK (longueur", token.length, ")");

// ── 2. Scénario de test : tâche complexe qui échoue ──────────────────────────
const REQUEST = "Implémente un système de cache Redis distribué avec TTL adaptatif, support des transactions MULTI/EXEC, failover automatique sur 3 nœuds, et sérialisation MessagePack";
const SUMMARY = "J'ai tenté de me connecter à redis://localhost:6379 via ioredis. Ensuite j'ai essayé d'écrire le module de failover mais je n'ai pas réussi à implémenter la logique de reconnexion automatique correctement — le heartbeat ne detecte pas les nœuds tombés.";
const ERROR   = "Error: connect ECONNREFUSED 127.0.0.1:6379\n    at TCPConnectWrap.afterConnect [as oncomplete] (net.js:1148:16)\nRedis connection timeout after 5000ms";

const prompt =
  `Un agent de code vient d'échouer sur une tâche. Déduis-en UNE leçon réutilisable pour ` +
  `ne pas répéter l'erreur, formulée comme un skill. Réponds STRICTEMENT en JSON sur une ligne, ` +
  `sans texte autour, au format : ` +
  `{"scope":"global|project","name":"nom-kebab-case","description":"quand utiliser ce skill","body":"instructions en Markdown"}. ` +
  `Mets scope="project" si la leçon est spécifique à ce dépôt, sinon "global". ` +
  `Demande de l'utilisateur : ${REQUEST}\nActions / réponse de l'agent : ${SUMMARY}\nErreur rencontrée : ${ERROR}`;

const msg = JSON.stringify({
  type: "user",
  message: { role: "user", content: [{ type: "text", text: prompt }] }
});

// ── 3. Appel Claude CLI (même flags que learn.rs) ─────────────────────────────
console.log("\n🚀 Appel Claude CLI (haiku, bypassPermissions)...");
const start = Date.now();

const child = spawn("claude", [
  "-p",
  "--input-format", "stream-json",
  "--output-format", "stream-json",
  "--verbose",
  "--permission-mode", "bypassPermissions",
  "--model", "haiku",
], {
  env: { ...process.env, CLAUDE_CODE_OAUTH_TOKEN: token, ANTHROPIC_API_KEY: undefined },
  stdio: ["pipe", "pipe", "pipe"],
});

child.stdin.write(msg + "\n");
child.stdin.end();

// ── 4. Parser la réponse stream-json ─────────────────────────────────────────
let resultText = "";
let buffer = "";

child.stdout.on("data", (chunk) => {
  buffer += chunk.toString();
  const lines = buffer.split("\n");
  buffer = lines.pop(); // garde la ligne incomplète
  for (const line of lines) {
    if (!line.trim()) continue;
    try {
      const v = JSON.parse(line);
      if (v.type === "result" && v.result) {
        resultText = v.result;
      }
    } catch { /* ligne non-JSON (logs verbose) */ }
  }
});

child.stderr.on("data", () => {}); // ignore stderr

await new Promise((resolve) => child.on("close", resolve));
const elapsed = ((Date.now() - start) / 1000).toFixed(1);
console.log(`⏱  Réponse reçue en ${elapsed}s`);

if (!resultText) {
  console.error("❌ Pas de résultat de Claude");
  process.exit(1);
}
console.log("\n📄 Texte brut :", resultText.slice(0, 300));

// ── 5. Extraire le JSON (même logique que learn.rs) ──────────────────────────
const start_ = resultText.indexOf("{");
const end_   = resultText.lastIndexOf("}");
if (start_ === -1 || end_ <= start_) {
  console.error("❌ Pas de JSON trouvé dans la réponse");
  process.exit(1);
}
const jsonStr = resultText.slice(start_, end_ + 1);
let parsed;
try {
  parsed = JSON.parse(jsonStr);
} catch (e) {
  console.error("❌ JSON invalide :", e.message);
  console.error("   Extrait :", jsonStr.slice(0, 200));
  process.exit(1);
}
console.log("\n✅ JSON parsé :");
console.log("   scope      :", parsed.scope);
console.log("   name       :", parsed.name);
console.log("   description:", parsed.description?.slice(0, 80));
console.log("   body       :", parsed.body?.slice(0, 80), "...");

// ── 6. Valider les champs ─────────────────────────────────────────────────────
const errors = [];
if (!["global", "project"].includes(parsed.scope)) errors.push(`scope invalide : "${parsed.scope}"`);
if (!parsed.name || parsed.name.length > 100 || /[/\\.]/.test(parsed.name)) errors.push(`name invalide : "${parsed.name}"`);
if (!parsed.description) errors.push("description vide");
if (!parsed.body) errors.push("body vide");

if (errors.length) {
  console.error("\n❌ Champs invalides :");
  errors.forEach(e => console.error("  -", e));
  process.exit(1);
}
console.log("\n✅ Tous les champs valides");

// ── 7. Écrire le SKILL.md (préfixé TEST_ pour ne pas polluer) ────────────────
const skillName = "test-hermes-" + parsed.name.slice(0, 30);
const skillDir  = join(SKILLS_DIR, skillName);
mkdirSync(skillDir, { recursive: true });
const content = `---\nname: ${skillName}\ndescription: ${parsed.description}\n---\n\n${parsed.body}\n`;
const skillPath = join(skillDir, "SKILL.md");
writeFileSync(skillPath, content, "utf8");

console.log("\n✅ SKILL.md écrit :", skillPath);
console.log("\n── Contenu ───────────────────────────────────────────────────────");
console.log(content);
console.log("──────────────────────────────────────────────────────────────────");
console.log("\n🎉 Test E2E réussi !");
