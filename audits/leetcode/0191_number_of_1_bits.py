# LeetCode 191: Number Of 1 Bits
# Python version

def hammingWeight(n: int) -> int:
    res = 0
    while n:
        n &= n - 1
        res += 1
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
