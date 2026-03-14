
# LeetCode 283: Move Zeroes
# Python version

def moveZeroes(nums: list[int]) -> None:
    """
    Do not return anything, modify nums in-place instead.
    """
    slow = 0
    for fast in range(len(nums)):
        
        if nums[fast] != 0 and nums[slow] == 0:
            nums[slow], nums[fast] = nums[fast], nums[slow]

        if nums[slow] != 0:
            slow += 1


def main():
    arg0 = [0, 1, 0, 3, 12]
    _result = moveZeroes(arg0)
    assert arg0 == [1, 3, 12, 0, 0]
    arg0 = [0]
    _result = moveZeroes(arg0)
    assert arg0 == [0]

if __name__ == "__main__":
    main()
