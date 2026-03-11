# LeetCode 392: Is Subsequence
# Python version with test cases

def isSubsequence(s: str, t: str) -> bool:
    si: int = 0
    ti: int = 0
    while si < len(s) and ti < len(t):
        if s[si] == t[ti]:
            si += 1
        ti += 1
    return si == len(s)

def main():
    print(isSubsequence("abc", "ahbgdc"))
    print(isSubsequence("axc", "ahbgdc"))
    print(isSubsequence("", "ahbgdc"))
    print(isSubsequence("ace", "abcde"))

if __name__ == "__main__":
    main()
