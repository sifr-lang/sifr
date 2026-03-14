# LeetCode 139: Word Break
# Python version

def wordBreak(s: str, wordDict: List[str]) -> bool:

    dp = [False] * (len(s) + 1)
    dp[len(s)] = True

    for i in range(len(s) - 1, -1, -1):
        for w in wordDict:
            if (i + len(w)) <= len(s) and s[i : i + len(w)] == w:
                dp[i] = dp[i + len(w)]
            if dp[i]:
                break

    return dp[0]



def main():
    assert wordBreak("leetcode", ["leet","code"]) == True
    assert wordBreak("applepenapple", ["apple","pen"]) == True

if __name__ == "__main__":
    main()
