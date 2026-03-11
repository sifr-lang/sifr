# LeetCode 201: Bitwise And Of Numbers Range
# Python version

def rangeBitwiseAnd(left: int, right: int) -> int:
    res = 0

    for i in range(32):
        bit = (left >> i) & 1
        if not bit:
            continue
        
        remain = left % (1 << (i + 1))
        diff = (1 << (i + 1)) - remain
        if right - left < diff:
            res = res | (1 << i)
    return res

# find the longest matching prefix of set bits between left and right

def rangeBitwiseAnd(left: int, right: int) -> int:
    i = 0
    while left != right:
        left = left >> 1
        right = right >> 1
        i += 1
    return left << i



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
