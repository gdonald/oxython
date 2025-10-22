"""
Demonstrates basic Python class and function behavior.

This example defines a simple `Greeter` class with an instance method,
uses a standalone utility function, and prints the results.
"""


class Greeter:
    def __init__(self, name):
        self.name = name

    def greeting(self):
        return f"Hello, {self.name}!"


def main():
    greeter = Greeter("Ada")
    message = greeter.greeting()

    print(message)


if __name__ == "__main__":
    main()
