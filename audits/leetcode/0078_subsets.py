# LeetCode 78: Subsets
# Python version

def subsets(nums: List[int]) -> List[List[int]]:
    res = []

    subset = []

    def dfs(i):
        if i >= len(nums):
            res.append(subset.copy())
            return
        # decision to include nums[i]
        subset.append(nums[i])
        dfs(i + 1)
        # decision NOT to include nums[i]
        subset.pop()
        dfs(i + 1)

    dfs(0)
    return res



def main():
    assert subsets([1,2,3]) == [[1, 2, 3], [1, 2], [1, 3], [1], [2, 3], [2], [3], []]

if __name__ == "__main__":
    main()
