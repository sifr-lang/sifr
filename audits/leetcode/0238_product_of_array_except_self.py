
# LeetCode 238: Product of Array Except Self
# Python version with test cases

def productExceptSelf(nums: list[int]) -> list[int]:
    n: int = len(nums)
    result: list[int] = []
    for i in range(n):
        result.append(1)

    # Left pass
    left: int = 1
    for i in range(n):
        result[i] = left
        left = left * nums[i]

    # Right pass
    right: int = 1
    i: int = n - 1
    while i >= 0:
        result[i] = result[i] * right
        right = right * nums[i]
        i -= 1

    return result

def main():
    assert productExceptSelf([1, 2, 3, 4]) == [24, 12, 8, 6]
    assert productExceptSelf([-1, 1, 0, -3, 3]) == [0, 0, 9, 0, 0]
    assert productExceptSelf([2, 3]) == [3, 2]

if __name__ == "__main__":
    main()
