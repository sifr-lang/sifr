# LeetCode 416: Partition Equal Subset Sum
# Python version

def canPartition(nums: List[int]) -> bool:
    if sum(nums) % 2:
        return False

    dp = set()
    dp.add(0)
    target = sum(nums) // 2

    for i in range(len(nums) - 1, -1, -1):
        nextDP = set()
        for t in dp:
            if (t + nums[i]) == target:
                return True
            nextDP.add(t + nums[i])
            nextDP.add(t)
        dp = nextDP
    return False



def main():
    print(canPartition([1,5,11,5]))
    print(canPartition([1,2,3,5]))

if __name__ == "__main__":
    main()
