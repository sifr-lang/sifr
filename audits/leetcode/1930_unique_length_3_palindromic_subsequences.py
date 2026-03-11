# LeetCode 1930: Unique Length 3 Palindromic Subsequences
# Python version

def countPalindromicSubsequence(s: str) -> int:
    count = 0
    chars = set(s)
    for char in chars:
        first,last = s.find(char),s.rfind(char)
        count += len(set(s[first+1:last]))
    return count


def main():
    print("no test cases")

if __name__ == "__main__":
    main()
