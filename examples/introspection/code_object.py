def greet(name):
    return "Hello, " + name

def calculate(x, y):
    return x + y

print("greet.__code__ =", greet.__code__)
print("calculate.__code__ =", calculate.__code__)
