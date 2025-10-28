def greet(name: str, age: int) -> str:
    return "Hello, " + name

def add(a: int, b: int) -> int:
    return a + b

def multiply(x: float, y: float) -> float:
    return x * y

message = greet("Bob", 25)
print(message)

sum_result = add(10, 20)
print("10 + 20 =", sum_result)

product = multiply(3.5, 2.0)
print("3.5 * 2.0 =", product)
