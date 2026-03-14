
# LeetCode 191: Number Of 1 Bits
# Python version

def hammingWeight(n: int) -> int:
    res = 0
    while n:
        n &= n - 1
        res += 1
    return res



def main():
    assert hammingWeight(11) == 3
    assert hammingWeight(128) == 1
    assert hammingWeight(2147483645) == 30

if __name__ == "__main__":
    main()
