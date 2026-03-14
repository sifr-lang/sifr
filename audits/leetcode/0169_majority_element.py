
# LeetCode 169: Majority Element (Boyer-Moore Voting)
# Python version with test cases

def majorityElement(nums: list[int]) -> int:
    candidate: int = nums[0]
    count: int = 0
    for i in range(len(nums)):
        if count == 0:
            candidate = nums[i]
        if nums[i] == candidate:
            count += 1
        else:
            count -= 1
    return candidate

def main():
    assert majorityElement([3, 2, 3]) == 3
    assert majorityElement([2, 2, 1, 1, 1, 2, 2]) == 2
    assert majorityElement([1]) == 1
    assert majorityElement([6, 5, 5]) == 5

if __name__ == "__main__":
    main()
