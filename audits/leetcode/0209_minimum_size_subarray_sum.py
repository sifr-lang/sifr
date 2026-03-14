# LeetCode 209: Minimum Size Subarray Sum
# Python version

def minSubArrayLen(target: int, nums: List[int]) -> int:
    res = float('inf')
    l, total = 0, 0

    for r in range(len(nums)):
        total += nums[r]
        while total >= target:
            res = min(res, r - l + 1)
            total -= nums[l]
            l += 1
    return res if res != float('inf') else 0

    


        



def main():
    assert minSubArrayLen(7, [2,3,1,2,4,3]) == 2
    assert minSubArrayLen(4, [1,4,4]) == 1
    assert minSubArrayLen(11, [1,1,1,1,1,1,1,1]) == 0

if __name__ == "__main__":
    main()
