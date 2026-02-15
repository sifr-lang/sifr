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
    print(permute([1,2,3]))

if __name__ == "__main__":
    main()
