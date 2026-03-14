
# LeetCode 509: Fibonacci Number
# Python version with test cases

def fib(n: int) -> int:
    if n <= 1:
        return n
    a: int = 0
    b: int = 1
    for i in range(2, n + 1):
        temp: int = b
        b = a + b
        a = temp
    return b

def main():
    assert fib(0) == 0
    assert fib(1) == 1
    assert fib(2) == 1
    assert fib(3) == 2
    assert fib(4) == 3
    assert fib(10) == 55
    assert fib(20) == 6765

if __name__ == "__main__":
    main()
