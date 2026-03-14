
# LeetCode 136: Single Number
# Python version

def singleNumber(nums: list[int]) -> int:
    res = 0
    for n in nums:
        res = n ^ res
    return res



def main():
    assert singleNumber([2,2,1]) == 1
    assert singleNumber([4,1,2,1,2]) == 4

if __name__ == "__main__":
    main()
