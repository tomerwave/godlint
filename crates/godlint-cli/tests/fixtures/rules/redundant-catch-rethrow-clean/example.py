def run():
    try:
        work()
    except Exception as error:
        log(error)
