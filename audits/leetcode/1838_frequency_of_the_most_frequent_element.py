# LeetCode 1838: Frequency Of The Most Frequent Element
# Python version

def maxFrequency(nums: List[int], k: int) -> int:
    nums.sort()

    l, r = 0, 0
    res, total = 0, 0

    while r < len(nums):
        total += nums[r]
        while nums[r] * (r - l + 1) > total + k:
            total -= nums[l]
            l += 1
        res = max(res, r - l + 1)
        r += 1
    
    return res
    



def main():
    assert maxFrequency([1, 2, 4], 5) == 3
    assert maxFrequency([1, 4, 8, 13], 5) == 2
    assert maxFrequency([3, 9, 6], 2) == 1

if __name__ == "__main__":
    main()
