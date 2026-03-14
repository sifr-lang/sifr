# LeetCode 39: Combination Sum
# Python version

def combinationSum(candidates: List[int], target: int) -> List[List[int]]:
    res = []

    def dfs(i, cur, total):
        if total == target:
            res.append(cur.copy())
            return
        if i >= len(candidates) or total > target:
            return

        cur.append(candidates[i])
        dfs(i, cur, total + candidates[i])
        cur.pop()
        dfs(i + 1, cur, total)

    dfs(0, [], 0)
    return res



def main():
    assert combinationSum([2,3,6,7], 7) == [[2, 2, 3], [7]]

if __name__ == "__main__":
    main()
