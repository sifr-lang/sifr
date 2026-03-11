# LeetCode 34: Find First And Last Position Of Element In Sorted Array
# Python version

def searchRange(nums: List[int], target: int) -> List[int]:
    left = binSearch(nums, target, True)
    right = binSearch(nums, target, False)
    return [left, right]

# leftBias=[True/False], if false, res is rightBiased

def binSearch(nums, target, leftBias):
    l, r = 0, len(nums) - 1
    i = -1
    while l <= r:
        m = (l + r) // 2
        if target > nums[m]:
            l = m + 1
        elif target < nums[m]:
            r = m - 1
        else:
            i = m
            if leftBias:
                r = m - 1
            else:
                l = m + 1
    return i



def main():
    print(searchRange([5,7,7,8,8,10], 8))
    print(searchRange([5,7,7,8,8,10], 6))

if __name__ == "__main__":
    main()
