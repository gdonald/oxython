total = -1
doubles = [1, 2, 3]


def stats(n):
    total = 0
    doubles = [i * 2 for i in range(0, n)]
    for value in doubles:
        total = total + value
    print(total)
    print(doubles)


stats(5)
