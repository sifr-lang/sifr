# LeetCode 268: Missing Number
# Python version

def missingNumber(nums: List[int]) -> int:
    res = len(nums)

    for i in range(len(nums)):
        res += i - nums[i]
    return res



def main():
    print(missingNumber([3,0,1]))
    print(missingNumber([0,1]))

if __name__ == "__main__":
    main()
