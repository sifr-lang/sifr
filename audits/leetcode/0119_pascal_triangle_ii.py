# LeetCode 119: Pascal Triangle Ii
# Python version

Memo = {}


def getRow(rowIndex: int) -> List[int]:
    if rowIndex in Memo:
        return Memo[rowIndex]
    if rowIndex == 0:
        return [1]
    ListPrec = getRow(rowIndex - 1)
    Result = [1]
    for i in range(0, len(ListPrec) - 1):
        Result.append(ListPrec[i] + ListPrec[i + 1])
    Result.append(1)
    Memo[rowIndex] = Result
    return Result



def main():
    assert getRow(3) == [1, 3, 3, 1]
    assert getRow(0) == [1]

if __name__ == "__main__":
    main()
