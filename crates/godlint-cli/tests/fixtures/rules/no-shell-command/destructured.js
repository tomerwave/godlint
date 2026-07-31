import { exec } from "node:child_process";

export function deploy(branch) {
  exec(`git checkout ${branch}`);
}
