# LeetCode 304: Range Sum Query 2D Immutable
# Python version

class NumMatrix:
    def __init__(self, matrix):
        self.sum_ = [[0] * (len(matrix[0]) + 1) for _ in range(len(matrix) + 1)]
        for i, line in enumerate(matrix):
            previous = 0
            for j, num in enumerate(line):
                previous += num
                above = self.sum_[i][j + 1]
                self.sum_[i + 1][j + 1] = previous + above
    def sumRegion(self, row1, col1, row2, col2):
        sum_col2 = self.sum_[row2 + 1][col2 + 1] - self.sum_[row1][col2 + 1]
        sum_col1 = self.sum_[row2 + 1][col1] - self.sum_[row1][col1]
        return sum_col2 - sum_col1

def main():
    obj = NumMatrix([[3, 0, 1, 4, 2], [5, 6, 3, 2, 1], [1, 2, 0, 1, 5], [4, 1, 0, 1, 7], [1, 0, 3, 0, 5]])
    assert obj.sumRegion(2, 1, 4, 3) == 8
    assert obj.sumRegion(1, 1, 2, 2) == 11
    assert obj.sumRegion(1, 2, 2, 4) == 12

if __name__ == "__main__":
    main()
