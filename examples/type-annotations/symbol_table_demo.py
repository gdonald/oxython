counter: int = 0
name: str = "Symbol Table Demo"
pi: float = 3.14159
active: bool = True

temp = 100
message = "Hello"

def process_data(x: int, y: int) -> int:
    local_sum: int = x + y
    local_product: int = x * y
    local_temp = local_sum + local_product
    return local_temp

counter = counter + 1
counter = counter + 1

result = process_data(10, 20)
print("Global counter:", counter)
print("Global name:", name)
print("Global pi:", pi)
print("Global active:", active)
print("Result from process_data:", result)

counter = 999
print("Counter after reassignment:", counter)
