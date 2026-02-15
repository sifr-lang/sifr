# LeetCode 1049: Last Stone Weight Ii
# Python version

def lastStoneWeightII(stones: List[int]) -> int:
    # Memoization
    stoneSum = sum(stones)
    target = ceil(stoneSum / 2)

    def dfs(i, total):
        if total >= target or i == len(stones):
            return abs(total - (stoneSum - total))
        if (i, total) in dp:
            return dp[(i, total)]

        dp[(i, total)] = min(dfs(i + 1, total),
                             dfs(i + 1, total + stones[i]))
        return dp[(i, total)]

    dp = {}
    return dfs(0, 0)



def main():
    print(lastStoneWeightII([2,7,4,1,8,1]))

if __name__ == "__main__":
    main()
