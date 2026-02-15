# LeetCode 1572: Matrix Diagonal Sum
# Python version

def PrimeSum(mat):
    cnt = 0
    for i in range(len(mat)):
        cnt += mat[i][i]
    return cnt


def CrossSum(mat):
    cnt = 0
    for i in range(len(mat)):
        cnt += mat[i][len(mat) - i - 1]
    return cnt


def diagonalSum(mat: List[List[int]]) -> int:
    prime = PrimeSum(mat)
    cross = CrossSum(mat)

    if len(mat) % 2 == 0:
        return prime + cross
    else:
        mid = len(mat) // 2
        mid_ele = mat[mid][mid]
        return prime + cross - mid_ele



def main():
    print(diagonalSum([[1,2,3],[4,5,6],[7,8,9]]))

if __name__ == "__main__":
    main()
