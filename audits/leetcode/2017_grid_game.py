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
    assert gridGame([[2, 5, 4], [1, 5, 1]]) == 4
    assert gridGame([[3, 3, 1], [8, 5, 2]]) == 4
    assert gridGame([[1, 3, 1, 15], [1, 3, 3, 1]]) == 7

if __name__ == "__main__":
    main()
