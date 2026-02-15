# LeetCode 168: Excel Sheet Column Title
# Python version

def convertToTitle(columnNumber: int) -> str:
    # Time: O(logn) - Log base 26 of n
    res = ""
    while columnNumber > 0:
        remainder = (columnNumber - 1) % 26
        res += chr(ord('A') + remainder)
        columnNumber = (columnNumber - 1) // 26

    return res[::-1] # reverse output



def main():
    print(convertToTitle(1))
    print(convertToTitle(28))
    print(convertToTitle(701))

if __name__ == "__main__":
    main()
