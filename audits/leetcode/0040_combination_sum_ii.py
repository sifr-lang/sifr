# LeetCode 40: Combination Sum Ii
# Python version

def combinationSum2(candidates: List[int], target: int) -> List[List[int]]:
    candidates.sort()

    res = []

    def backtrack(cur, pos, target):
        if target == 0:
            res.append(cur.copy())
            return
        if target <= 0:
            return

        prev = -1
        for i in range(pos, len(candidates)):
            if candidates[i] == prev:
                continue
            cur.append(candidates[i])
            backtrack(cur, i + 1, target - candidates[i])
            cur.pop()
            prev = candidates[i]

    backtrack([], 0, target)
    return res



def main():
    assert combinationSum2([10,1,2,7,6,1,5], 8) == [[1, 1, 6], [1, 2, 5], [1, 7], [2, 6]]

if __name__ == "__main__":
    main()
