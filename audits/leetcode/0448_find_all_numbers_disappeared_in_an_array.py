# LeetCode 448: Find All Numbers Disappeared In An Array
# Python version

def findDisappearedNumbers(nums: List[int]) -> List[int]:
    for n in nums:
        i = abs(n) - 1
        nums[i] = -1 * abs(nums[i])

    res = []
    for i, n in enumerate(nums):
        if n > 0:
            res.append(i + 1)
    return res



def main():
    assert findDisappearedNumbers([4, 3, 2, 7, 8, 2, 3, 1]) == [5, 6]
    assert findDisappearedNumbers([1, 1]) == [2]

if __name__ == "__main__":
    main()
