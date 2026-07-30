const cp = require("child_process");

function deploy(branch) {
  cp.exec(`git checkout ${branch}`);
}
