# LeetCode 1523: Count Odd Numbers In An Interval Range
# Python version

def countOdds(low: int, high: int) -> int:
    if low%2!=0 or high%2!=0:
        return (high-low)//2 +1
    return (high-low)//2



def main():
    print(countOdds(3, 7))
    print(countOdds(8, 10))

if __name__ == "__main__":
    main()
