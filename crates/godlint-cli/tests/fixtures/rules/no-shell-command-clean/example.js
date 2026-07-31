import { execFile } from "node:child_process";

export function deploy(branch) {
  execFile("git", ["checkout", branch]);
}
