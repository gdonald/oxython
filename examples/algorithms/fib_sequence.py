count = 8
numbers = [0, 1]

while len(numbers) < count:
    numbers.append(numbers[-1] + numbers[-2])

print(numbers[:count])
