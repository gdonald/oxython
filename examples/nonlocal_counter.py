def make_counter():
    count = 0

    def increment():
        nonlocal count
        count = count + 1
        print(count)

    def get_count():
        print(count)

    def reset():
        nonlocal count
        count = 0

    increment()
    increment()
    increment()
    get_count()
    reset()
    get_count()
    increment()


make_counter()
