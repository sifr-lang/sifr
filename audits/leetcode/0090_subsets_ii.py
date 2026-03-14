# LeetCode 90: Subsets Ii
# Python version

def subsetsWithDup(nums: List[int]) -> List[List[int]]:
    res = []
    nums.sort()

    def backtrack(i, subset):
        if i == len(nums):
            res.append(subset[::])
            return

        # All subsets that include nums[i]
        subset.append(nums[i])
        backtrack(i + 1, subset)
        subset.pop()
        # All subsets that don't include nums[i]
        while i + 1 < len(nums) and nums[i] == nums[i + 1]:
            i += 1
        backtrack(i + 1, subset)

    backtrack(0, [])
    return res



def main():
    assert subsetsWithDup([1,2,2]) == [[1, 2, 2], [1, 2], [1], [2, 2], [2], []]

if __name__ == "__main__":
    main()
