# LeetCode 1498: Number Of Subsequences That Satisfy The Given Sum Condition
# Python version

def numSubseq(nums: List[int], target: int) -> int:
    nums.sort()

    res, mod = 0, (10**9 + 7)

    left, right = 0, len(nums) - 1
    while  left <= right:
        if (nums[left] + nums[right]) > target:
            right -= 1
        else:
            res += 1 << (right - left)
            left += 1
    return res % mod



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
