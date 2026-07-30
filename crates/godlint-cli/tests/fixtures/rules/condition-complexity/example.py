def example(a, b, c, cond):
    if a and b or c:
        pass
    if (a if cond else b) and c:
        pass
