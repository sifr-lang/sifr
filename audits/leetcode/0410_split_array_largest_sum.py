# LeetCode 410: Split Array Largest Sum
# Python version

def splitArray(nums: List[int], m: int) -> int:
    def canSplit(largest):
        subarray = 0
        curSum = 0
        for n in nums:
            curSum += n
            if curSum > largest:
                subarray += 1
                curSum = n
        return subarray + 1 <= m

    l, r = max(nums), sum(nums)
    res = r
    while l <= r:
        mid = l + ((r - l) // 2)
        if canSplit(mid):
            res = mid
            r = mid - 1
        else:
            l = mid + 1
    return res



def main():
    print(splitArray([7,2,5,10,8], 2))

if __name__ == "__main__":
    main()
