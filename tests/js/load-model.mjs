// Loads omarchy/Model.js the way QML sees it: a plain script whose top-level
// functions become the library's API. `.pragma library` is QML-only syntax,
// so it is dropped before node evaluates the file.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const modelPath = fileURLToPath(new URL("../../omarchy/Model.js", import.meta.url));

export function loadModel() {
  const source = readFileSync(modelPath, "utf8").replace(/^\.pragma library\s*$/m, "");
  const context = vm.createContext({});
  vm.runInContext(source, context, { filename: modelPath });
  return context;
}
