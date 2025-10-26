class Greeter:
    def __init__(self, name):
        self.name = name

    def greeting(self):
        return "Hello, " + self.name + "!"


greeter = Greeter("Ada")
message = greeter.greeting()
print(message)
