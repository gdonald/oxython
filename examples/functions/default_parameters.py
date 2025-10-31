def greet(name, greeting="Hello", punctuation="!"):
    return greeting + " " + name + punctuation

print(greet("Alice", "Hi", "!!!"))
print(greet("Bob", "Hey"))
print(greet("Charlie"))

def create_message(username, role="user"):
    return username + ":" + role

print(create_message("admin"))
print(create_message("john", "moderator"))

def power(base, exponent=2):
    result = 1
    i = 0
    while i < exponent:
        result = result * base
        i = i + 1
    return result

print(power(3))
print(power(3, 3))
print(power(2, 5))

def format_price(price, currency="USD"):
    return currency + " " + price

print(format_price("100"))
print(format_price("50", "EUR"))
