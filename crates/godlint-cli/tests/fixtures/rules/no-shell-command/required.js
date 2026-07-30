const { execSync } = require("child_process");

function deploy(branch) {
  execSync(`git push ${branch}`);
}
