"""A file that satisfies every configured rule."""


def total(values):
    return sum(value for value in values if value > 0)
