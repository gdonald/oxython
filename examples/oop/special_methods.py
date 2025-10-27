class Book:
    def __init__(self, title):
        self.title = title

    def __str__(self):
        return "Book: " + self.title

    def __repr__(self):
        return "Book(title=" + self.title + ")"

class Magazine:
    def __init__(self, name):
        self.name = name

    def __repr__(self):
        return "Magazine: " + self.name

class SimpleIterator:
    def __init__(self):
        self.value = 0

    def __iter__(self):
        return self

    def __next__(self):
        return "next_value"

book = Book("Python Guide")
print(book)

magazine = Magazine("Tech Monthly")
print(magazine)

it = SimpleIterator()
iterator = it.__iter__()
print(iterator.__next__())
