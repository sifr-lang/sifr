
# LeetCode 70: Climbing Stairs
# Python version with test cases

def climbStairs(n: int) -> int:
    if n <= 2:
        return n
    a: int = 1
    b: int = 2
    for i in range(3, n + 1):
        temp: int = b
        b = a + b
        a = temp
    return b

def main():
    assert climbStairs(1) == 1
    assert climbStairs(2) == 2
    assert climbStairs(3) == 3
    assert climbStairs(5) == 8
    assert climbStairs(10) == 89

if __name__ == "__main__":
    main()
