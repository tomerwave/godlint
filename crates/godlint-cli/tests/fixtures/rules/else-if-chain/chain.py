def accepted(value):
    if value == 1:
        return 10
    elif value == 2:
        return 20
    elif value == 3:
        return 30
    return 0


def reported(value):
    if value == 1:
        for item in items:
            work(item)
