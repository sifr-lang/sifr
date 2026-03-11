# LeetCode 448: Find All Numbers Disappeared In An Array
# Python version

def findDisappearedNumbers(nums: List[int]) -> List[int]:
    for n in nums:
        i = abs(n) - 1
        nums[i] = -1 * abs(nums[i])

    res = []
    for i, n in enumerate(nums):
        if n > 0:
            res.append(i + 1)
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
