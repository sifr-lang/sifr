# LeetCode 930: Binary Subarrays With Sum
# Python version

def numSubarraysWithSum(nums: List[int], goal: int) -> int:

    def helper(x):
        if x < 0: return 0

        res = 0
        l, cur = 0, 0
        for r in range(len(nums)):
            cur += nums[r]
            while cur > x:
                cur -= nums[l]
                l += 1
            res += (r - l + 1)
        return res
    
    return helper(goal) - helper(goal - 1)



def main():
    assert numSubarraysWithSum([1,0,1,0,1], 2) == 4

if __name__ == "__main__":
    main()
