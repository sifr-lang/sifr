# LeetCode 46: Permutations
# Python version

def permute(nums: List[int]) -> List[List[int]]:
    res = []

    # base case
    if len(nums) == 1:
        return [nums[:]]  # nums[:] is a deep copy

    for i in range(len(nums)):
        n = nums.pop(0)
        perms = permute(nums)

        for perm in perms:
            perm.append(n)
        res.extend(perms)
        nums.append(n)
    return res



def main():
    assert permute([1,2,3]) == [[3, 2, 1], [2, 3, 1], [1, 3, 2], [3, 1, 2], [2, 1, 3], [1, 2, 3]]

if __name__ == "__main__":
    main()
