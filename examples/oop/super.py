class Animal:
    def __init__(self, name):
        self.name = name
    
    def speak(self):
        return "Some sound"
    
    def describe(self):
        return "I am an animal"

class Dog(Animal):
    def __init__(self, name, breed):
        super().__init__(name)
        self.breed = breed
    
    def speak(self):
        parent_sound = super().speak()
        return parent_sound + " -> Woof!"
    
    def describe(self):
        parent_desc = super().describe()
        return parent_desc + " named " + self.name

dog = Dog("Buddy", "Golden Retriever")
print(dog.name)
print(dog.breed)
print(dog.speak())
print(dog.describe())
