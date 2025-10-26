text = "racecar"
chars = []

for char in text:
    chars.append(char)

reversed_text = ""

for char in chars:
    reversed_text = char + reversed_text

is_palindrome = text == reversed_text
print(is_palindrome)
