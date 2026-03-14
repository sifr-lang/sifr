
# LeetCode 374: Guess Number Higher Or Lower
# Python version

PICK = 6

def guess(num: int) -> int:
    if num == PICK:
        return 0
    if num < PICK:
        return 1
    return -1

def guessNumber(n: int) -> int:
    # return a num btw 1,..,n
    low = 1
    high = n

    while low <= high:
        mid = low + (high - low) // 2
        myGuess = guess(mid)
        if myGuess == 1:
            low = mid + 1
        elif myGuess == -1:
            high = mid - 1
        else:
            return mid
    return -1



def main():
    assert guessNumber(10) == 6
    assert guessNumber(6) == 6

if __name__ == "__main__":
    main()
