# LeetCode 918: Maximum Sum Circular Subarray
# Python version

def maxSubarraySumCircular(nums: List[int]) -> int:
    globMax, globMin = nums[0], nums[0]
    curMax, curMin = 0, 0
    total = 0
    
    for i, n in enumerate(nums):
        curMax = max(curMax + n, n)
        curMin = min(curMin + n, n)
        total += n
        globMax = max(curMax, globMax)
        globMin = min(curMin, globMin)

    return max(globMax, total - globMin) if globMax > 0 else globMax



def main():
    print(maxSubarraySumCircular([1,-2,3,-2]))
    print(maxSubarraySumCircular([5,-3,5]))
    print(maxSubarraySumCircular([-3,-2,-3]))

if __name__ == "__main__":
    main()
