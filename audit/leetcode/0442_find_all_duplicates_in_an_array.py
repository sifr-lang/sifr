# LeetCode 442: Find All Duplicates In An Array
# Python version

def findDuplicates(nums: List[int]) -> List[int]:
    res = []

    for n in nums:
        n = abs(n)
        if nums[n - 1] < 0:
            res.append(n)
        nums[n - 1] = -nums[n - 1]
    
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
