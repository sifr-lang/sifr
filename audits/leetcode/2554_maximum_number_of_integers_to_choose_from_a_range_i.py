# LeetCode 2554: Maximum Number Of Integers To Choose From A Range I
# Python version

def maxCount(banned: List[int], n: int, maxSum: int) -> int:
    nums = {x:1 for x in range(1, n + 1)} # hashmap for storing the required elements
    for i in banned:
        if nums.get(i):
            del nums[i]
    sum = 0
    count = 0
    for i in nums:
        sum += i
        if sum <= maxSum:
            count += 1
        else:
            break
    return count



def main():
    assert maxCount(6, [1, 6, 5], 2) == 3

if __name__ == "__main__":
    main()
