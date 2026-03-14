
# LeetCode 202: Happy Number
# Python version

def isHappy(n: int) -> bool:
    slow, fast = n, sumSquareDigits(n)

    while slow != fast:
        fast = sumSquareDigits(fast)
        fast = sumSquareDigits(fast)
        slow = sumSquareDigits(slow)

    return True if fast == 1 else False


def sumSquareDigits(n):
    output = 0
    while n:
        output += (n % 10) ** 2
        n = n // 10
    return output



def main():
    assert isHappy(19) == True
    assert isHappy(2) == False

if __name__ == "__main__":
    main()
