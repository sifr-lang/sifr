import heapq

# LeetCode 2971: Find Polygon With The Largest Perimeter
# Python version

def largestPerimeter(nums: list[int]) -> int:
    nums.sort()
    res = -1
    total = 0

    for n in nums:
        if total > n:
            res = total + n
        total += n
    
    return res

# Time complexity O(n + 30logn) ~ O(n)

def largestPerimeter(nums: list[int]) -> int:
    curSum = sum(nums)
    heapq._heapify_max(nums)

    while nums and curSum <= nums[0] * 2:
        curSum -= heapq._heappop_max(nums)
        
    return curSum if len(nums) > 2 else -1
    



def main():
    assert largestPerimeter([5,5,5]) == 15
    assert largestPerimeter([1,12,1,2,5,50,3]) == 12

if __name__ == "__main__":
    main()
