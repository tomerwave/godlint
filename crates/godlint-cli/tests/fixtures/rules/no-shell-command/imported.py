from os import system


def deploy(branch):
    system(f"git push {branch}")
