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
    print(isPalindrome(121))
    print(isPalindrome(-121))
    print(isPalindrome(10))
    print(isPalindrome(0))
    print(isPalindrome(12321))

if __name__ == "__main__":
    main()
