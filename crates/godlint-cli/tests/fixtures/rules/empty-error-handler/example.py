try:
    work()
except ValueError:
    pass
try:
    work()
except Exception as error:
    ...
try:
    work()
except Exception:
    recover()
