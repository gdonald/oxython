def double(n):
    return n * 2


def apply_twice(value):
    return double(double(value))


def factorial(n):
    if n < 2:
        return 1
    return n * factorial(n - 1)


result = apply_twice(4)
print("apply_twice(4) =", result)
print("factorial(6) =", factorial(6))
