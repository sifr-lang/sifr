# LeetCode 50: Powx N
# Python version

def myPow(x: float, n: int) -> float:
    def helper(x, n):
        if x == 0:
            return 0
        if n == 0:
            return 1

        res = helper(x * x, n // 2)
        return x * res if n % 2 else res

    res = helper(x, abs(n))
    return res if n >= 0 else 1 / res



def main():
    assert myPow(2.0, 10) == 1024.0
    assert myPow(2.0, -2) == 0.25

if __name__ == "__main__":
    main()
