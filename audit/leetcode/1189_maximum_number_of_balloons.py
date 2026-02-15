# LeetCode 1189: Maximum Number Of Balloons
# Python version

def maxNumberOfBalloons(text: str) -> int:
    countText = Counter(text)
    balloon = Counter("balloon")

    res = len(text)  # or float("inf")
    for c in balloon:
        res = min(res, countText[c] // balloon[c])
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
