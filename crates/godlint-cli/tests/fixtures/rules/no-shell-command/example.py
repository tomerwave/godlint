import os
import subprocess


def deploy(branch):
    subprocess.run(f"git checkout {branch}", shell=True)
    os.system(f"git push {branch}")
