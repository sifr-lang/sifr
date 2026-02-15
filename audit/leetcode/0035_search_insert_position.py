# LeetCode 35: Search Insert Position
# Python version

def searchInsert(nums: List[int], target: int) -> int:
    # O(log n) and O(1)
    
    
    low, high = 0, len(nums)
    while low<high:
        mid = low +(high - low) // 2
        if target > nums[mid]:
            low = mid + 1
        else:
            high = mid
    return low



def main():
    print(searchInsert([1,3,5,6], 5))
    print(searchInsert([1,3,5,6], 2))
    print(searchInsert([1,3,5,6], 7))

if __name__ == "__main__":
    main()
