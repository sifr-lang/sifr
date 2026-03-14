
# LeetCode 442: Find All Duplicates In An Array
# Python version

def findDuplicates(nums: list[int]) -> list[int]:
    res = []

    for n in nums:
        n = abs(n)
        if nums[n - 1] < 0:
            res.append(n)
        nums[n - 1] = -nums[n - 1]
    
    return res



def main():
    assert findDuplicates([4, 3, 2, 7, 8, 2, 3, 1]) == [2, 3]
    assert findDuplicates([1, 1, 2]) == [1]
    assert findDuplicates([1]) == []

if __name__ == "__main__":
    main()
