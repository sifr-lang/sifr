
# LeetCode 64: Minimum Path Sum
# Python version

def minPathSum(grid: list[list[int]]) -> int:
    m, n = len(grid), len(grid[0])
    prev = [float("inf")] * n
    prev[-1] = 0

    for row in range(m - 1, -1, -1):
        dp = [float("inf")] * n
        for col in range(n - 1, -1, -1):
            if col < n - 1:
                dp[col] = min(dp[col], dp[col + 1])
            dp[col] = min(dp[col], prev[col]) + grid[row][col]
        prev = dp

    return prev[0]



def main():
    assert minPathSum([[1,3,1],[1,5,1],[4,2,1]]) == 7

if __name__ == "__main__":
    main()
