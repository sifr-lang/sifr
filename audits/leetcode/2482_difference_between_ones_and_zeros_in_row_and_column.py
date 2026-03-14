
# LeetCode 2482: Difference Between Ones And Zeros In Row And Column
# Python version

def onesMinusZeros(grid: list[list[int]]) -> list[list[int]]:
    m , n = len(grid), len(grid[0])
    rowCount = [[0, 0] for _ in range(m)] # (zeros, ones)
    colCount = [[0, 0] for _ in range(n)]
    res = []
    for r in range(m):
        for c in range(n):
            if grid[r][c] == 1:
                rowCount[r][1] += 1
                colCount[c][1] += 1
            else:
                rowCount[r][0] += 1
                colCount[c][0] += 1
    for r in range(m):
        row =[]
        for c in range(n):
            row.append(rowCount[r][1] + colCount[c][1] - 
                        rowCount[r][0] - colCount[c][0])
        res.append(row)
    return res

    



def main():
    assert onesMinusZeros([[0, 1, 1], [1, 0, 1], [0, 0, 1]]) == [[0, 0, 4], [0, 0, 4], [-2, -2, 2]]
    assert onesMinusZeros([[1, 1, 1], [1, 1, 1]]) == [[5, 5, 5], [5, 5, 5]]

if __name__ == "__main__":
    main()
