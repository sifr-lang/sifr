# LeetCode 125: Valid Palindrome
# Python version

def isPalindrome(s: str) -> bool:
    new = ''
    for a in s:
        if a.isalpha() or a.isdigit():
            new += a.lower()
    return (new == new[::-1])



def main():
    print(isPalindrome("A man, a plan, a canal: Panama"))
    print(isPalindrome("race a car"))

if __name__ == "__main__":
    main()
