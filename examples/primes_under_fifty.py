limit = 50
primes = []

for candidate in range(2, limit):
    is_prime = True
    for prime in primes:
        if candidate % prime == 0:
            is_prime = False
            break
    if is_prime:
        primes.append(candidate)

print(primes)
