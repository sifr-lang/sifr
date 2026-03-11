# LeetCode 136: Single Number
# Python version

def singleNumber(nums: List[int]) -> int:
    res = 0
    for n in nums:
        res = n ^ res
    return res



def main():
    print(singleNumber([2,2,1]))
    print(singleNumber([4,1,2,1,2]))

if __name__ == "__main__":
    main()
