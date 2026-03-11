# LeetCode 2017: Grid Game
# Python version

def gridGame(grid):
    result = float("inf")
    left, right = 0, sum(grid[0])

    for a, b in zip(grid[0], grid[1]):
        right -= a
        result = min(result, max(left, right))
        left += b
    return result



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
