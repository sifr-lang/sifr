
# LeetCode 1260: Shift 2D Grid
# Python version

def shiftGrid(grid: list[list[int]], k: int) -> list[list[int]]:
    M, N = len(grid), len(grid[0])
    
    def posToVal(r, c):
        return r * N + c
    def valToPos(v):
        return [v // N, v % N] # r, c
    
    res = [[0] * N for i in range(M)]
    for r in range(M):
        for c in range(N):
            newVal = (posToVal(r, c) + k) % (M * N)
            newR, newC = valToPos(newVal)
            res[newR][newC] = grid[r][c]
    return res



def main():
    assert shiftGrid([[1, 2, 3], [4, 5, 6], [7, 8, 9]], 1) == [[9, 1, 2], [3, 4, 5], [6, 7, 8]]
    assert shiftGrid([[3, 8, 1, 9], [19, 7, 2, 5], [4, 6, 11, 10], [12, 0, 21, 13]], 4) == [[12, 0, 21, 13], [3, 8, 1, 9], [19, 7, 2, 5], [4, 6, 11, 10]]
    assert shiftGrid([[1, 2, 3], [4, 5, 6], [7, 8, 9]], 9) == [[1, 2, 3], [4, 5, 6], [7, 8, 9]]

if __name__ == "__main__":
    main()
