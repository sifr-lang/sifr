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
    assert maxNumberOfBalloons('nlaebolko') == 1
    assert maxNumberOfBalloons('loonbalxballpoon') == 2
    assert maxNumberOfBalloons('leetcode') == 0

if __name__ == "__main__":
    main()
