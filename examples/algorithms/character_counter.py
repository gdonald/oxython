text = "banana"
counts = {}

for char in text:
    if char in counts:
        counts[char] += 1
    else:
        counts[char] = 1

print(counts)
