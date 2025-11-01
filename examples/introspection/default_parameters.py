def no_defaults(a, b):
    return a + b

def with_defaults(a, b=10, c=20):
    return a + b + c

def all_defaults(x=1, y=2, z=3):
    return x + y + z

def greet(name, greeting="Hello", punctuation="!"):
    return greeting + " " + name + punctuation

print(no_defaults.__name__)
print(no_defaults.__defaults__)

print(with_defaults.__name__)
print(with_defaults.__defaults__)

print(all_defaults.__name__)
print(all_defaults.__defaults__)

print(greet.__name__)
print(greet.__defaults__)
