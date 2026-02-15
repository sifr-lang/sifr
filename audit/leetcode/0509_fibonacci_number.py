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
    print(fib(0))
    print(fib(1))
    print(fib(2))
    print(fib(3))
    print(fib(4))
    print(fib(10))
    print(fib(20))

if __name__ == "__main__":
    main()
