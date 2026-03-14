
# LeetCode 48: Rotate Image
# Python version

def rotate(matrix: list[list[int]]) -> None:
    """
    Do not return anything, modify matrix in-place instead.
    """
    l, r = 0, len(matrix) - 1
    while l < r:
        for i in range(r - l):
            top, bottom = l, r

            # save the topleft
            topLeft = matrix[top][l + i]

            # move bottom left into top left
            matrix[top][l + i] = matrix[bottom - i][l]

            # move bottom right into bottom left
            matrix[bottom - i][l] = matrix[bottom][r - i]

            # move top right into bottom right
            matrix[bottom][r - i] = matrix[top + i][r]

            # move top left into top right
            matrix[top + i][r] = topLeft
        r -= 1
        l += 1



def main():
    arg0 = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    _result = rotate(arg0)
    assert arg0 == [[7, 4, 1], [8, 5, 2], [9, 6, 3]]
    arg0 = [[5, 1, 9, 11], [2, 4, 8, 10], [13, 3, 6, 7], [15, 14, 12, 16]]
    _result = rotate(arg0)
    assert arg0 == [[15, 13, 2, 5], [14, 3, 4, 1], [12, 6, 8, 9], [16, 7, 10, 11]]

if __name__ == "__main__":
    main()
