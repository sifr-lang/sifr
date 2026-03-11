# LeetCode 26: Remove Duplicates From Sorted Array
# Python version

def removeDuplicates(nums: List[int]) -> int:
    L = 1
    
    for R in range(1, len(nums)):
        if nums[R] != nums[R - 1]:
            nums[L] = nums[R]
            L += 1
    return L



def main():
    print(removeDuplicates([1,1,2]))
    print(removeDuplicates([0,0,1,1,1,2,2,3,3,4]))

if __name__ == "__main__":
    main()
