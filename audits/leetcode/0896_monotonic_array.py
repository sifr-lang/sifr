# LeetCode 896: Monotonic Array
# Python version

def isMonotonic(nums: List[int]) -> bool:
    increasing = decreasing = True
    
    for i in range(len(nums) - 1):
        if nums[i] > nums[i + 1]:
            increasing = False
        if nums[i] < nums[i + 1]:
            decreasing = False
    
    return increasing or decreasing



def main():
    print(isMonotonic([1,2,2,3]))
    print(isMonotonic([6,5,4,4]))
    print(isMonotonic([1,3,2]))

if __name__ == "__main__":
    main()
