numbers: list = [1, 2, 3, 4, 5]
print("numbers =", numbers)

scores: dict = {"Alice": 90, "Bob": 85}
print("scores =", scores)

def process_list(items: list) -> int:
    return len(items)

def get_score(grades: dict, name: str) -> int:
    return grades[name]

print("length of numbers:", process_list(numbers))
print("Alice's score:", get_score(scores, "Alice"))
