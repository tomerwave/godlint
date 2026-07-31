import subprocess


def deploy(branch):
    subprocess.run(["git", "checkout", branch])
    subprocess.run(["git", "push", branch], shell=False)
