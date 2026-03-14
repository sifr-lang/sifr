# LeetCode 5: Longest Palindromic Substring
# Python version

def longestPalindrome(s: str) -> str:
    res = ""
    lenres = 0
    for i in range(len(s)):
        s1 = helper(s, i, i)
        s2 = helper(s, i, i + 1)
    return s2
    

def helper(s, left, right):
        while left >= 0 and right < len(s) and s[left] == s[right]:
            if (right - left + 1) > lenres:
                res = s[left:right+1]
                lenres = right - left + 1
            left -= 1
            right += 1
        return res





def main():
    assert longestPalindrome("babad",) == 'bab'
    assert longestPalindrome("cbbd",) == 'bb'

if __name__ == "__main__":
    main()
