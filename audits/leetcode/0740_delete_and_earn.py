
# LeetCode 740: Delete And Earn
# Python version

def deleteAndEarn(nums):
    """
    :type nums: list[int]
    :rtype: int
    """

    upperLimit = max(nums) + 1 
    store = [0] * upperLimit

    for num in nums:
        store[num] += num

    dp = [0] * upperLimit

    dp[1] = 1 * store[1]
    for i in range(2, upperLimit):
        dp[i] = max(dp[i - 2] + store[i], dp[i - 1])

    return dp[-1]


def main():
    assert deleteAndEarn([3,4,2]) == 6
    assert deleteAndEarn([2,2,3,3,3,4]) == 9

if __name__ == "__main__":
    main()
