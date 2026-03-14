# LeetCode 213: House Robber Ii
# Python version

def rob(nums: List[int]) -> int:
    return max(nums[0], helper(nums[1:]), helper(nums[:-1]))


def helper(nums):
    rob1, rob2 = 0, 0

    for n in nums:
        newRob = max(rob1 + n, rob2)
        rob1 = rob2
        rob2 = newRob
    return rob2



def main():
    assert rob([2,3,2]) == 3
    assert rob([1,2,3,1]) == 4

if __name__ == "__main__":
    main()
