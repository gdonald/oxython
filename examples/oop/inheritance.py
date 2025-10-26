class Animal:
    def __init__(self, name):
        self.name = name

    def speak(self):
        return "Some sound"

    def introduce(self):
        return self.name + " says " + self.speak()


class Dog(Animal):
    def speak(self):
        return "Woof!"


class Cat(Animal):
    def speak(self):
        return "Meow!"


dog = Dog("Buddy")
cat = Cat("Whiskers")

print(dog.introduce())
print(cat.introduce())
