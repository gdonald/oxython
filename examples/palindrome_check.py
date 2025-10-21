text = "Never odd or even"
cleaned = "".join(char.lower() for char in text if char.isalnum())
is_palindrome = cleaned == cleaned[::-1]

print(is_palindrome)
