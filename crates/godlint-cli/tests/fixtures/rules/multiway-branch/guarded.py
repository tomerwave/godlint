def guarded(value, flag, other):
    match value:
        case 1 if flag:
            return 10
        case 2 if other:
            return 20
        case 3 if flag and other:
            return 30
        case _:
            return 0
