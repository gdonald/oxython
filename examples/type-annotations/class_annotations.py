class Person:
    def __init__(self, name: str, age: int):
        self.name = name
        self.age = age

    def get_info(self) -> str:
        return self.name

class Calculator:
    def add(self, a: int, b: int) -> int:
        return a + b

    def multiply(self, x: float, y: float) -> float:
        return x * y

person: Person = Person("Alice", 30)
print(person.get_info())

calc: Calculator = Calculator()
result = calc.add(5, 10)
print("5 + 10 =", result)

product = calc.multiply(2.5, 4.0)
print("2.5 * 4.0 =", product)
