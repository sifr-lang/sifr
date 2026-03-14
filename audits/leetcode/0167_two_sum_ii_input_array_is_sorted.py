# LeetCode 167: Two Sum Ii Input Array Is Sorted
# Python version

def twoSum(numbers: List[int], target: int) -> List[int]:
    l, r = 0, len(numbers) - 1

    while l < r:
        curSum = numbers[l] + numbers[r]

        if curSum > target:
            r -= 1
        elif curSum < target:
            l += 1
        else:
            return [l + 1, r + 1]



def main():
    assert twoSum([2,7,11,15], 9) == [1, 2]
    assert twoSum([2,3,4], 6) == [1, 3]

if __name__ == "__main__":
    main()
