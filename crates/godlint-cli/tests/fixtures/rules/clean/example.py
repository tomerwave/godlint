"""A file that satisfies every configured rule."""


def total(values):
    """Sums the positive values."""
    return sum(value for value in values if value > 0)
