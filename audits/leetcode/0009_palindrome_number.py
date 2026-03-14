# LeetCode 9: Palindrome Number
# Python version with test cases

def isPalindrome(x: int) -> bool:
    if x < 0:
        return False
    rev: int = 0
    original: int = x
    while x > 0:
        rev = rev * 10 + x % 10
        x = x // 10
    return original == rev

def main():
    assert isPalindrome(121) == True
    assert isPalindrome(-121) == False
    assert isPalindrome(10) == False
    assert isPalindrome(0) == True
    assert isPalindrome(12321) == True

if __name__ == "__main__":
    main()
