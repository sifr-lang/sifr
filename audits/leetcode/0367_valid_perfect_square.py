
# LeetCode 367: Valid Perfect Square
# Python version

def isPerfectSquare(num: int) -> bool:
    for i in range(1, num+1):
        if i * i == num:
            return True
        if i* i > num:
            return False




def isPerfectSquare_2(num: int) -> bool:
    l ,r = 1, num
    while l <= r:
        mid = (l +r) // 2
        if mid * mid > num:
            r = mid - 1
        elif mid * mid < num:
            l = mid + 1
        else:
            return True
    return False



def main():
    assert isPerfectSquare(16) == True
    assert isPerfectSquare(14) == False

if __name__ == "__main__":
    main()
