# LeetCode 118: Pascals Triangle
# Python version

def generate(rowIndex) -> List[List[int]]:
    if rowIndex == 0:
        return [[1]]
    else:
        return getAllRow(rowIndex - 1)


def getAllRow(rowIndex):
    if rowIndex == 0:
        return [[1]]
    ListPrec = getAllRow(rowIndex - 1)
    Len = len(ListPrec[-1])
    ListPrec.append([1])
    for i in range(0, Len - 1):
        ListPrec[-1].append(ListPrec[-2][i] + ListPrec[-2][i + 1])
    ListPrec[-1].append(1)
    return ListPrec



def main():
    print(generate(5))

if __name__ == "__main__":
    main()
