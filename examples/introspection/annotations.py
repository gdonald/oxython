def add(x: int, y: int) -> int:
    return x + y

def greet(name: str) -> str:
    return "Hello, " + name

def process(data: str, count: int, flag: bool):
    return data

print("add.__annotations__ =", add.__annotations__)
print("greet.__annotations__ =", greet.__annotations__)
print("process.__annotations__ =", process.__annotations__)

def no_annotations(a, b):
    return a + b

print("no_annotations.__annotations__ =", no_annotations.__annotations__)
