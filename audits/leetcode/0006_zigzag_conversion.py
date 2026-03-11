# LeetCode 6: Zigzag Conversion
# Python version

def convert(s: str, numRows: int) -> str:
    if numRows == 1 or numRows >= len(s):
        return s

    res = [""] * numRows

    index = 0
    step = 1
    for c in s:
        res[index] += c
        if index == 0:
            step = 1
        elif index == numRows - 1:
            step = -1
        index += step

    return "".join(res)


def main():
    print(convert("PAYPALISHIRING", 3))
    print(convert("A", 1))

if __name__ == "__main__":
    main()
