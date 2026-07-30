const child_process = require("child_process");

function deploy(branch) {
  child_process.exec(`git checkout ${branch}`);
}
