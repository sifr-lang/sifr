# LeetCode 287: Find The Duplicate Number
# Python version

def findDuplicate(nums: List[int]) -> int:
    slow, fast = 0, 0
    while True:
        slow = nums[slow]
        fast = nums[nums[fast]]
        if slow == fast:
            break

    slow2 = 0
    while True:
        slow = nums[slow]
        slow2 = nums[slow2]
        if slow == slow2:
            return slow



def main():
    print(findDuplicate([1,3,4,2,2]))
    print(findDuplicate([3,1,3,4,2]))

if __name__ == "__main__":
    main()
