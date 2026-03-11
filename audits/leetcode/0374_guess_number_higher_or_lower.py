# LeetCode 374: Guess Number Higher Or Lower
# Python version

def guessNumber(n: int) -> int:
    # return a num btw 1,..,n
    
    low = 1
    high = n
    
    while True:
        mid = low + (high - low) // 2
        myGuess = guess(mid)
        if myGuess == 1:
            low = mid + 1
        elif myGuess == -1:
            high = mid - 1
        else:
            return mid



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
