# LeetCode 704: Binary Search
# Python version with test cases

def search(nums: list[int], target: int) -> int:
    left: int = 0
    right: int = len(nums) - 1
    while left <= right:
        mid: int = left + (right - left) // 2
        if nums[mid] == target:
            return mid
        elif nums[mid] < target:
            left = mid + 1
        else:
            right = mid - 1
    return -1

def main():
    print(search([-1, 0, 3, 5, 9, 12], 9))
    print(search([-1, 0, 3, 5, 9, 12], 2))
    print(search([5], 5))
    print(search([2, 5], 5))

if __name__ == "__main__":
    main()
