# LeetCode 300: Longest Increasing Subsequence
# Python version with test cases (O(n^2) DP approach)

def lengthOfLIS(nums: list[int]) -> int:
    n: int = len(nums)
    if n == 0:
        return 0
    dp: list[int] = []
    for i in range(n):
        dp.append(1)
    for i in range(1, n):
        for j in range(i):
            if nums[j] < nums[i]:
                if dp[j] + 1 > dp[i]:
                    dp[i] = dp[j] + 1
    best: int = 0
    for i in range(n):
        if dp[i] > best:
            best = dp[i]
    return best

def main():
    print(lengthOfLIS([10, 9, 2, 5, 3, 7, 101, 18]))
    print(lengthOfLIS([0, 1, 0, 3, 2, 3]))
    print(lengthOfLIS([7, 7, 7, 7, 7, 7, 7]))
    print(lengthOfLIS([1]))

if __name__ == "__main__":
    main()
