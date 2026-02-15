# LeetCode 1137: N Th Tribonacci Number
# Python version

Memo = {}


def tribonacci(n: int):
    if n in Memo:
        return Memo[n]
    if n == 0:
        return 0
    if n == 1:
        return 1
    if n == 2:
        return 1
    Memo[n] = (
        tribonacci(n - 1) + tribonacci(n - 2) + tribonacci(n - 3)
    )
    return Memo[n]



def main():
    print(tribonacci(4))
    print(tribonacci(25))

if __name__ == "__main__":
    main()
