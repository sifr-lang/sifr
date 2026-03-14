# LeetCode 1800: Maximum Ascending Subarray Sum
# Python version

def maxAscendingSum(nums: List[int]) -> int:
    curSum = results = nums[0]

    for i in range(1, len(nums)):
        if nums[i] <= nums[i - 1]:
            curSum = 0
        curSum += nums[i]
        results = max(curSum, results)

    return results



def main():
    assert maxAscendingSum([10,20,30,5,10,50]) == 65

if __name__ == "__main__":
    main()
