# LeetCode 53: Maximum Subarray (Kadane's Algorithm)
# Python version with test cases

def maxSubArray(nums: list[int]) -> int:
    max_sum: int = nums[0]
    current_sum: int = nums[0]
    for i in range(1, len(nums)):
        if current_sum < 0:
            current_sum = nums[i]
        else:
            current_sum = current_sum + nums[i]
        if current_sum > max_sum:
            max_sum = current_sum
    return max_sum

def main():
    print(maxSubArray([-2, 1, -3, 4, -1, 2, 1, -5, 4]))
    print(maxSubArray([1]))
    print(maxSubArray([5, 4, -1, 7, 8]))
    print(maxSubArray([-1]))
    print(maxSubArray([-2, -1]))

if __name__ == "__main__":
    main()
