# LeetCode 215: Kth Largest Element In An Array
# Python version

def findKthLargest(nums: List[int], k: int) -> int:
    heapify(nums)
    while len(nums) > k:
        heappop(nums)
    return nums[0]

# Solution: Sorting
# Time Complexity:
#   - Best Case: O(n)
#   - Average Case: O(n*log(n))
#   - Worst Case:O(n*log(n))
# Extra Space Complexity: O(n)

def findKthLargest(nums: List[int], k: int) -> int:
    nums.sort()
    return nums[len(nums) - k]


# Solution: QuickSelect
# Time Complexity: O(n)
# Extra Space Complexity: O(n)

def findKthLargest(nums: List[int], k: int) -> int:
    pivot = random.choice(nums)
    left = [num for num in nums if num > pivot]
    mid = [num for num in nums if num == pivot]
    right = [num for num in nums if num < pivot]

    length_left = len(left)
    length_right = len(right)
    length_mid = len(mid)
    if k <= length_left:
        return findKthLargest(left, k)
    elif k > length_left + length_mid:
        return findKthLargest(right, k - length_mid - length_left)
    else:
        return mid[0]



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
