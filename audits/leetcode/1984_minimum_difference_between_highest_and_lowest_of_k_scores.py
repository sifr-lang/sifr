
# LeetCode 1984: Minimum Difference Between Highest And Lowest Of K Scores
# Python version

def minimumDifference(nums: list[int], k: int) -> int:
    nums.sort()
    l, r = 0, k - 1
    res = float("inf")
    
    while r < len(nums):
        res = min(res, nums[r] - nums[l])
        l, r = l + 1, r + 1
    return res



def main():
    assert minimumDifference([90], 1) == 0

if __name__ == "__main__":
    main()
