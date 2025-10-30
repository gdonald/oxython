global_var = 42

def simple_func():
    return 1

print(simple_func.__name__)
print(simple_func.__qualname__)
print(simple_func.__closure__)

def outer(x):
    y = 10

    def inner(z):
        return x + y + z

    print(inner.__name__)
    print(inner.__qualname__)
    print(inner.__closure__)

    return inner(5)

result = outer(3)
print(result)
