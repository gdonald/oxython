def greet(name):
    return "Hello, " + name

def calculate(x, y):
    return x + y

print("greet.__name__ =", greet.__name__)
print("calculate.__name__ =", calculate.__name__)
print("greet.__doc__ =", greet.__doc__)
print("calculate.__doc__ =", calculate.__doc__)
