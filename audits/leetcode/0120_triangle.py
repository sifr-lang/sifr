
# LeetCode 120: Triangle
# Python version

def minimumTotal(triangle: list[list[int]]) -> int:
    dp = triangle[-1]

    for row in range(len(triangle) - 2, -1, -1):
        for col in range(0, row + 1):
            dp[col] = triangle[row][col] + min(dp[col], dp[col + 1])

    return dp[0]



def main():
    assert minimumTotal([[2],[3,4],[6,5,7],[4,1,8,3]]) == 11

if __name__ == "__main__":
    main()
