class Counter:
    def __init__(self):
        self.count = 0

    def __iter__(self):
        return self

    def __next__(self):
        return "next"

counter = Counter()
iterator = counter.__iter__()
print(iterator.__next__())
