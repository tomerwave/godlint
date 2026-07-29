try:
    work()
except:
    raise
try:
    work()
except ValueError as error:
    report(error)
