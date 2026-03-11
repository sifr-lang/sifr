# LeetCode 75: Sort Colors
# Python version

def sortColors(nums: List[int]) -> None:
    low = 0
    high = len(nums) - 1
    mid = 0

    while mid <= high:
        if nums[mid] == 0:
            nums[low], nums[mid] = nums[mid], nums[low]
            low += 1
            mid += 1
        elif nums[mid] == 1:
            mid +=1
        else:
            nums[mid], nums[high] = nums[high], nums[mid]
            high -= 1
    return nums



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
