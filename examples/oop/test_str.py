class Person:
    def __init__(self, name):
        self.name = name

    def __str__(self):
        return "Person: " + self.name

person = Person("Alice")
print(person)
