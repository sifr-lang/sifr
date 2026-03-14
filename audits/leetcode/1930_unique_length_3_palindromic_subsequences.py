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
    assert countPalindromicSubsequence('aabca') == 3
    assert countPalindromicSubsequence('adc') == 0
    assert countPalindromicSubsequence('bbcbaba') == 4

if __name__ == "__main__":
    main()
