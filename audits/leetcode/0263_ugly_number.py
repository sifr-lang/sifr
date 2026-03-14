
# LeetCode 263: Ugly Number
# Python version

def isUgly(n: int) -> bool:
    if n <= 0:
        return False
    
    for p in [2, 3, 5]:
        while n % p == 0:
            n = n // p
    return n == 1



def main():
    assert isUgly(6) == True
    assert isUgly(1) == True
    assert isUgly(14) == False

if __name__ == "__main__":
    main()
