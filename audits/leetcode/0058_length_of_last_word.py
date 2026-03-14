# LeetCode 58: Length of Last Word
# Python version with test cases

def lengthOfLastWord(s: str) -> int:
    length: int = 0
    i: int = len(s) - 1
    # Skip trailing spaces
    while i >= 0 and s[i] == " ":
        i -= 1
    # Count last word characters
    while i >= 0 and s[i] != " ":
        length += 1
        i -= 1
    return length

def main():
    assert lengthOfLastWord("Hello World") == 5
    assert lengthOfLastWord("   fly me   to   the moon  ") == 4
    assert lengthOfLastWord("luffy is still joyboy") == 6
    assert lengthOfLastWord("a") == 1

if __name__ == "__main__":
    main()
