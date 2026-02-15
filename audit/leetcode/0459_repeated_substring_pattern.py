# LeetCode 459: Repeated Substring Pattern
# Python version

def repeatedSubstringPattern(s: str) -> bool:
    return s in (s + s)[1:-1]
    



def main():
    print(repeatedSubstringPattern("abab"))
    print(repeatedSubstringPattern("aba"))

if __name__ == "__main__":
    main()
