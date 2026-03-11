# LeetCode 1: Two Sum
# Python version

def twoSum(nums: List[int], target: int) -> List[int]:
    prevMap = {}  # val -> index

    for i, n in enumerate(nums):
        diff = target - n
        if diff in prevMap:
            return [prevMap[diff], i]
        prevMap[n] = i



def main():
    print(twoSum([2,7,11,15], 9))
    print(twoSum([3,2,4], 6))
    print(twoSum([3,3], 6))

if __name__ == "__main__":
    main()
