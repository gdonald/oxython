x: int = 100
y: str = "test"

z = 50
message = "hello"

def calculate(a: int, b) -> int:
    return a + b

def process(value):
    return value * 2

result1 = calculate(x, z)
print("calculate(100, 50) =", result1)

result2 = process(10)
print("process(10) =", result2)

print("x + z =", x + z)
print("y + message =", y + " " + message)
