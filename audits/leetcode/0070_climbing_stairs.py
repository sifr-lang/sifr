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
    print(climbStairs(1))
    print(climbStairs(2))
    print(climbStairs(3))
    print(climbStairs(5))
    print(climbStairs(10))

if __name__ == "__main__":
    main()
