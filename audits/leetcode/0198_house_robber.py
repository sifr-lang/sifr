# LeetCode 198: House Robber
# Python version with test cases

def rob(nums: list[int]) -> int:
    if len(nums) == 0:
        return 0
    if len(nums) == 1:
        return nums[0]
    prev2: int = nums[0]
    prev1: int = nums[1]
    if nums[0] > nums[1]:
        prev1 = nums[0]
    for i in range(2, len(nums)):
        current: int = prev2 + nums[i]
        if prev1 > current:
            current = prev1
        prev2 = prev1
        prev1 = current
    return prev1

def main():
    assert rob([1, 2, 3, 1]) == 4
    assert rob([2, 7, 9, 3, 1]) == 12
    assert rob([2, 1, 1, 2]) == 4
    assert rob([0]) == 0

if __name__ == "__main__":
    main()
