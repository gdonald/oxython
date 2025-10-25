temperatures = [68.5, 70.1, 69.8, 71.0]
total = 0

for reading in temperatures:
    total += reading

average = total / len(temperatures)

print(round(average, 1))
